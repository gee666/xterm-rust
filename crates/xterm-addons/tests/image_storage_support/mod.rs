use std::{
    any::Any,
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::Rc,
};
use xterm_addons::image::{storage::*, types::RgbaImage};
use xterm_core::headless::Terminal;

#[derive(Debug)]
pub struct Failure(pub &'static str);
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for Failure {}

pub struct Recorder {
    pub events: Rc<RefCell<Vec<String>>>,
    pub size: Cell<CellSize>,
    pub top: Cell<bool>,
    pub bottom: Cell<bool>,
    pub deny_top: Cell<bool>,
    pub extraction: RefCell<Option<ImageResource>>,
}
impl Recorder {
    fn new(events: Rc<RefCell<Vec<String>>>, size: CellSize) -> Self {
        Self {
            events,
            size: Cell::new(size),
            top: Cell::new(false),
            bottom: Cell::new(false),
            deny_top: Cell::new(false),
            extraction: RefCell::new(Some(raster(1, 1))),
        }
    }
    fn layer(&self, layer: ImageLayer) -> &Cell<bool> {
        match layer {
            ImageLayer::Top => &self.top,
            ImageLayer::Bottom => &self.bottom,
        }
    }
    fn log(&self, event: String) {
        self.events.borrow_mut().push(event);
    }
}
impl StorageRenderer for Recorder {
    fn cell_size(&self) -> Result<CellSize, StorageError> {
        Ok(self.size.get())
    }
    fn has_layer(&self, layer: ImageLayer) -> bool {
        self.layer(layer).get()
    }
    fn insert_layer(&self, layer: ImageLayer) -> Result<(), StorageError> {
        self.log(format!("insert:{layer:?}"));
        if layer != ImageLayer::Top || !self.deny_top.get() {
            self.layer(layer).set(true);
        }
        Ok(())
    }
    fn remove_layer(&self, layer: ImageLayer) -> Result<(), StorageError> {
        self.log(format!("remove:{layer:?}"));
        self.layer(layer).set(false);
        Ok(())
    }
    fn rescale_canvas(&self) -> Result<(), StorageError> {
        self.log("rescale".into());
        Ok(())
    }
    fn clear_all(&self, layer: Option<ImageLayer>) -> Result<(), StorageError> {
        self.log(format!("clear:{layer:?}"));
        Ok(())
    }
    fn clear_lines(&self, start: usize, end: usize) -> Result<(), StorageError> {
        self.log(format!("lines:{start}:{end}"));
        Ok(())
    }
    fn draw_placeholder(&self, col: usize, row: usize, count: usize) -> Result<(), StorageError> {
        self.log(format!("placeholder:{col}:{row}:{count}"));
        Ok(())
    }
    fn draw(
        &self,
        spec: &ImageSpec,
        tile: f64,
        col: usize,
        row: usize,
        count: usize,
    ) -> Result<(), StorageError> {
        self.log(format!(
            "draw:{}:{}:{col}:{row}:{count}",
            spec.id,
            number(tile)
        ));
        Ok(())
    }
    fn extract_tile(
        &self,
        spec: &ImageSpec,
        tile: f64,
    ) -> Result<Option<ImageResource>, StorageError> {
        self.log(format!("extract:{}:{}", spec.id, number(tile)));
        Ok(self.extraction.borrow().clone())
    }
    fn bitmap_to_canvas(&self, image: &ImageResource) -> Result<ImageResource, StorageError> {
        self.log("bitmap-to-canvas".into());
        Ok(raster(image.width()?, image.height()?))
    }
}
pub fn number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
pub fn raster(width: u32, height: u32) -> ImageResource {
    let count = usize::try_from(u64::from(width) * u64::from(height) * 4).unwrap();
    ImageResource::from_pixels(RgbaImage::new(width, height, vec![0; count]).unwrap())
}
struct Bitmap {
    width: Rc<Cell<u32>>,
    height: Rc<Cell<u32>>,
    label: &'static str,
    events: Rc<RefCell<Vec<String>>>,
    failure: Option<Rc<dyn Error>>,
}
impl ImageSurface for Bitmap {
    fn width(&self) -> Result<u32, StorageError> {
        Ok(self.width.get())
    }
    fn height(&self) -> Result<u32, StorageError> {
        Ok(self.height.get())
    }
    fn bitmap_close(&self) -> Result<Option<BitmapClose>, StorageError> {
        let (width, height, events, failure, label) = (
            self.width.clone(),
            self.height.clone(),
            self.events.clone(),
            self.failure.clone(),
            self.label,
        );
        Ok(Some(Rc::new(move || {
            events.borrow_mut().push(format!("close:{label}"));
            if let Some(error) = &failure {
                return Err(StorageError::Host(Rc::clone(error)));
            }
            width.set(0);
            height.set(0);
            Ok(())
        })))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
pub fn bitmap(
    label: &'static str,
    width: u32,
    height: u32,
    events: Rc<RefCell<Vec<String>>>,
    failure: Option<Rc<dyn Error>>,
) -> ImageResource {
    ImageResource::new(Bitmap {
        width: Rc::new(Cell::new(width)),
        height: Rc::new(Cell::new(height)),
        label,
        events,
        failure,
    })
}

pub struct Fixture {
    pub terminal: Rc<Terminal>,
    pub renderer: Rc<Recorder>,
    pub storage: ImageStorage,
    pub events: Rc<RefCell<Vec<String>>>,
}
impl Fixture {
    pub fn new(cols: usize, rows: usize, size: CellSize) -> Self {
        let terminal = Rc::new(Terminal::new(cols, rows, 5).unwrap());
        let events = Rc::new(RefCell::new(Vec::new()));
        let diagnostics = events.clone();
        let host = Rc::new(
            TerminalStorageHost::from_terminal(
                &terminal,
                Rc::new(move |level, message| {
                    diagnostics
                        .borrow_mut()
                        .push(format!("diagnostic:{level:?}:{message}"));
                    Ok(())
                }),
            )
            .unwrap(),
        );
        let renderer = Rc::new(Recorder::new(events.clone(), size));
        let storage =
            ImageStorage::new(host, renderer.clone(), ImageStorageOptions::default()).unwrap();
        Self {
            terminal,
            renderer,
            storage,
            events,
        }
    }
    pub fn cursor(&self) -> (usize, usize, usize) {
        let buffer = self
            .terminal
            .with_state(|state| state.active_buffer())
            .unwrap();
        let buffer = buffer.borrow();
        (buffer.x, buffer.y, buffer.ybase)
    }
    pub fn set_cursor(&self, x: usize, y: usize) {
        set_cursor(&self.terminal, x, y);
    }
    pub fn tile(&self, x: usize, y: usize) -> Option<Rc<ImageTileInfo>> {
        let buffer = self
            .terminal
            .with_state(|state| state.active_buffer())
            .unwrap();
        let line = buffer.borrow().lines.get(y)?;
        let payload = line.borrow().get_extended(x)?.payload()?;
        payload.downcast::<ImageTileInfo>().ok()
    }
    pub fn events(&self) -> Vec<String> {
        self.events.borrow().clone()
    }
}
pub fn set_cursor(terminal: &Terminal, x: usize, y: usize) {
    let buffer = terminal.with_state(|state| state.active_buffer()).unwrap();
    let mut buffer = buffer.borrow_mut();
    buffer.x = x;
    buffer.y = y;
}
