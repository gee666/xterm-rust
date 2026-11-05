// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::{image_canvas_failures::Patch, image_canvas_support::*};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{IipEffect, IipOptions},
    },
    NameDecodePolicy,
};
use xterm_web::image::{IipDecodedImage, IipImageDecoder};

const QOI: &[u8] = include_bytes!("../../../xterm-addons/tests/fixtures/qoi/palette.qoi");
const PNG: &[u8] =
    include_bytes!("../../../xterm-addons/tests/fixtures/image-metrics/w3c_home.png");

fn put(handler: &mut IipHandler, text: &str) {
    let units: Vec<_> = text.chars().map(u32::from).collect();
    handler.start();
    handler.put(&units, 0, units.len()).unwrap();
}
fn multipart(handler: &mut IipHandler) {
    let geometry = IipGeometry {
        columns: 80.0,
        rows: 24.0,
        render: None,
        device_pixel_ratio: None,
    };
    for text in ["MultipartFile=inline=1", "FilePart=AAAA"] {
        put(handler, text);
        assert!(handler.end(true, &geometry).unwrap().is_empty());
    }
}
fn sequence(bytes: &[u8], width: u32, height: u32) -> String {
    let binary: String = bytes.iter().map(|&byte| char::from(byte)).collect();
    let encoded = call(&js_sys::global(), "btoa", &[binary.into()])
        .as_string()
        .unwrap();
    format!("File=inline=1;width={width}px;height={height}px;preserveAspectRatio=0:{encoded}")
}

#[wasm_bindgen_test]
fn payload_release_matches_source_on_constructor_failure_and_reentry() {
    for (key, is_qoi, nested, throws) in [
        ("ImageData", true, false, true),
        ("ImageData", true, false, false),
        ("ImageData", true, true, true),
        ("ImageData", true, true, false),
        ("Blob", false, false, true),
        ("getContext", true, false, true),
        ("createImageBitmap", false, false, true),
        ("createImageBitmap", true, false, true),
    ] {
        // Resized QOI follows the bitmap path; other QOI cases stay synchronous.
        let width = if is_qoi && key != "createImageBitmap" {
            640
        } else {
            80
        };
        let height = if width == 640 { 80 } else { 40 };
        let text = sequence(if is_qoi { QOI } else { PNG }, width, height);
        let reference = js_sys::Function::new_with_args("text,key,nested,throws", "
          const states=[]; const failure={iipRelease:true};
          const h=new SourceIIPHandler({pixelLimit:16777216,iipSizeLimit:33554432},{},
            {addImage(){}},{cols:80,rows:24,_core:{}});
          function write(text){h.start();const units=Uint32Array.from(text,c=>c.charCodeAt(0));
            h.put(units,0,units.length);return h.end(true)}
          const target=key==='getContext'?HTMLCanvasElement.prototype:globalThis;
          const original=Object.getOwnPropertyDescriptor(target,key);
          const construct=key==='ImageData'||key==='Blob';
          Object.defineProperty(target,key,{configurable:true,writable:true,value:function(...args){
            states.push(h._dec.loadedBytes);
            if(nested){write('MultipartFile=inline=1');write('FilePart=AAAA');states.push(h._dec.loadedBytes)}
            if(throws)throw failure;
            return construct?Reflect.construct(original.value,args):Reflect.apply(original.value,this,args);
          }});
          let caught=false;
          try{const result=write(text);if(result instanceof Promise)throw Error('unexpected pending result')}
          catch(error){if(error!==failure)throw error;caught=true}
          finally{Object.defineProperty(target,key,original)}
          return {states,loaded:h._dec.loadedBytes,caught,multipart:h._isMultipart,
            aborted:h._aborted,abortMulti:h._abortMulti,parserState:h._hp.state};")
            .apply(&JsValue::UNDEFINED, &js_sys::Array::of4(&text.clone().into(), &key.into(), &nested.into(), &throws.into()))
            .unwrap();
        let handler = Rc::new(RefCell::new(
            IipHandler::new(IipOptions::default(), NameDecodePolicy::BrowserAtob).unwrap(),
        ));
        put(&mut handler.borrow_mut(), &text);
        let mut effects = handler
            .borrow_mut()
            .end(
                true,
                &IipGeometry {
                    columns: 80.0,
                    rows: 24.0,
                    render: None,
                    device_pixel_ratio: None,
                },
            )
            .unwrap();
        let IipEffect::Display(request) = effects.remove(0) else {
            panic!("expected image")
        };
        let observed = Rc::new(RefCell::new(Vec::new()));
        let captured = observed.clone();
        let live = handler.clone();
        let callback = Closure::<dyn FnMut()>::new(move || {
            captured.borrow_mut().push(live.borrow().loaded_bytes());
            if nested {
                multipart(&mut live.borrow_mut());
                captured.borrow_mut().push(live.borrow().loaded_bytes());
            }
        });
        let target = if key == "getContext" {
            eval("HTMLCanvasElement.prototype")
        } else {
            js_sys::global().into()
        };
        let original = get(&target, key);
        let failure = eval("({iipRelease:true})");
        let wrapper = js_sys::Function::new_with_args(
            "callback,original,throws,failure,construct",
            "
          return function(...args){callback();if(throws)throw failure;
            return construct?Reflect.construct(original,args):Reflect.apply(original,this,args)}",
        )
        .apply(
            &JsValue::UNDEFINED,
            &js_sys::Array::of5(
                callback.as_ref(),
                &original,
                &throws.into(),
                &failure,
                &(key == "ImageData" || key == "Blob").into(),
            ),
        )
        .unwrap();
        let patch = Patch::new(target, key, wrapper);
        let decoder = IipImageDecoder::default();
        let caught = match decoder.prepare_transfer(&request, &handler) {
            Ok(IipDecodedImage::Immediate(_)) => false,
            Ok(IipDecodedImage::Pending(_)) => panic!("unexpected pending result"),
            Err(error) => {
                assert!(error == failure, "{key}");
                true
            }
        };
        drop(patch);
        let expected: Vec<_> = js_sys::Array::from(&get(&reference, "states"))
            .iter()
            .map(|value| value.as_f64().unwrap() as usize)
            .collect();
        assert_eq!(
            *observed.borrow(),
            expected,
            "{key}, nested={nested}, throws={throws}"
        );
        assert_eq!(
            get(&reference, "loaded").as_f64(),
            Some(handler.borrow().loaded_bytes() as f64),
            "{key}, nested={nested}"
        );
        assert_eq!(get(&reference, "caught").as_bool(), Some(caught), "{key}");
        {
            let handler = handler.borrow();
            assert_eq!(
                get(&reference, "multipart").as_bool(),
                Some(handler.is_multipart())
            );
            assert_eq!(
                get(&reference, "aborted").as_bool(),
                Some(handler.is_aborted())
            );
            assert_eq!(
                get(&reference, "abortMulti").as_bool(),
                Some(handler.is_multipart_aborted())
            );
            assert_eq!(
                get(&reference, "parserState").as_f64(),
                Some(handler.header_parser().state as u8 as f64)
            );
        }
        if key == "ImageData" || key == "Blob" {
            assert!(expected[0] > 0);
        } else {
            assert_eq!(expected, [0]);
        }
        assert_eq!(
            handler.borrow().loaded_bytes(),
            if nested && throws {
                4
            } else if throws && (key == "ImageData" || key == "Blob") {
                expected[0]
            } else {
                0
            }
        );
    }
}
