// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::{image_canvas_failures::Patch, image_canvas_support::*};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{IipEffect, IipOptions},
    },
    NameDecodePolicy,
};

const PNG: &str = "iVBORw0KGgoAAAAASUhEUgAAAAIAAAABAAAAAAAAAAA=";
fn write(handler: &RefCell<IipHandler>, text: &str) -> Vec<IipEffect> {
    let data: Vec<_> = text.chars().map(u32::from).collect();
    let mut handler = handler.borrow_mut();
    handler.start();
    handler.put(&data, 0, data.len()).unwrap();
    handler
        .end(
            true,
            &IipGeometry {
                columns: 80.0,
                rows: 24.0,
                render: None,
                device_pixel_ratio: None,
            },
        )
        .unwrap()
}
fn snapshot(handler: &IipHandler, stage: u32) -> Vec<u32> {
    vec![
        stage,
        handler.loaded_bytes() as u32,
        handler.is_multipart() as u32,
        handler.is_multipart_aborted() as u32,
        handler.is_aborted() as u32,
        handler.header_parser().state as u32,
    ]
}

#[wasm_bindgen_test]
fn warning_getter_call_throw_and_live_release_match_unchanged_source() {
    for payload in ["A", "AAAA", PNG] {
        for mode in [
            "none",
            "getter-throw",
            "call-throw",
            "reset",
            "multipart",
            "multipart-throw",
            "getter-multipart",
            "getter-multipart-throw",
            "inline-disabled",
        ] {
            let text = format!("File=inline=1:{payload}");
            let reference = js_sys::Function::new_with_args(
                "text,mode",
                "
              const states=[],messages=[],failure={warning:true};
              const h=new SourceIIPHandler({pixelLimit:2,iipSizeLimit:33554432},{},
                {addImage(){throw Error('unexpected image')}},{cols:80,rows:24,_core:{}});
              function write(text){h.start();const data=Uint32Array.from(text,c=>c.charCodeAt(0));
                h.put(data,0,data.length);return h.end(true)}
              function snapshot(stage){states.push([stage,h._dec.loadedBytes,+h._isMultipart,
                +h._abortMulti,+h._aborted,h._hp.state])}
              function observe(stage){
                snapshot(stage);
                if(stage===+( !mode.startsWith('getter-') )){
                  if(mode==='reset')h.reset();
                  else if(mode.includes('multipart')){
                    write('MultipartFile=inline=1');write('FilePart=AQIDBAUG');
                  }else if(mode==='inline-disabled')write('File=inline=0:AAAA');
                  snapshot(stage+2);
                }
              }
              const original=Object.getOwnPropertyDescriptor(console,'warn');
              Object.defineProperty(console,'warn',{configurable:true,get(){
                observe(0);if(mode==='getter-throw'||mode==='getter-multipart-throw')throw failure;
                return function(message){if(this!==console)throw Error('wrong receiver');
                  messages.push(message);observe(1);
                  if(mode==='call-throw'||mode==='multipart-throw')throw failure;}
              }});
              let caught=false;
              try{if(write(text)!==true)throw Error('unexpected pending result')}
              catch(error){if(error!==failure)throw error;caught=true}
              finally{Object.defineProperty(console,'warn',original)}
              snapshot(4);return {states,messages,caught};
            ",
            )
            .call2(&JsValue::UNDEFINED, &text.clone().into(), &mode.into())
            .unwrap();
            let handler = Rc::new(RefCell::new(
                IipHandler::new(
                    IipOptions {
                        pixel_limit: 2.0,
                        ..IipOptions::default()
                    },
                    NameDecodePolicy::BrowserAtob,
                )
                .unwrap(),
            ));
            let effects = write(&handler, &text);
            let [IipEffect::Warning(warning)] = effects.as_slice() else {
                panic!("{effects:?}")
            };
            let states = Rc::new(RefCell::new(Vec::new()));
            let observed = states.clone();
            let live = handler.clone();
            let callback = Closure::<dyn FnMut(u32)>::new(move |stage| {
                observed.borrow_mut().push(snapshot(&live.borrow(), stage));
                let at_getter = mode.starts_with("getter-");
                if stage == u32::from(!at_getter) {
                    if mode == "reset" {
                        live.borrow_mut().reset();
                    } else if mode.contains("multipart") {
                        assert!(write(&live, "MultipartFile=inline=1").is_empty());
                        assert!(write(&live, "FilePart=AQIDBAUG").is_empty());
                    } else if mode == "inline-disabled" {
                        assert!(write(&live, "File=inline=0:AAAA").is_empty());
                    }
                    observed
                        .borrow_mut()
                        .push(snapshot(&live.borrow(), stage + 2));
                }
            });
            let messages = js_sys::Array::new();
            let failure = eval("({warning:true})");
            let descriptor = js_sys::Function::new_with_args(
                "callback,mode,messages,failure",
                "
              return {configurable:true,get(){
                callback(0);if(mode==='getter-throw'||mode==='getter-multipart-throw')throw failure;
                return function(message){if(this!==console)throw Error('wrong receiver');
                  messages.push(message);callback(1);
                  if(mode==='call-throw'||mode==='multipart-throw')throw failure;}
              }};
            ",
            )
            .apply(
                &JsValue::UNDEFINED,
                &js_sys::Array::of4(callback.as_ref(), &mode.into(), &messages, &failure),
            )
            .unwrap();
            let console = get(&js_sys::global(), "console");
            let patch = Patch::with_descriptor(console, "warn", descriptor);
            let result = IipHandler::report_warning(&handler, warning, |warning| {
                let console = js_sys::Reflect::get(&js_sys::global(), &"console".into())?;
                let method = js_sys::Reflect::get(&console, &"warn".into())?
                    .dyn_into::<js_sys::Function>()?;
                method.call1(&console, &warning.into()).map(|_| ())
            });
            drop(patch);
            let caught = match result {
                Ok(()) => false,
                Err(error) => {
                    assert!(error == failure, "{mode}");
                    true
                }
            };
            states.borrow_mut().push(snapshot(&handler.borrow(), 4));
            let actual = js_sys::Array::new();
            for row in states.borrow().iter() {
                actual.push(
                    &row.iter()
                        .map(|&v| JsValue::from(v))
                        .collect::<js_sys::Array>(),
                );
            }
            assert_eq!(
                js_sys::JSON::stringify(&actual).unwrap(),
                js_sys::JSON::stringify(&get(&reference, "states")).unwrap(),
                "{payload}, {mode}"
            );
            assert_eq!(
                js_sys::JSON::stringify(&messages).unwrap(),
                js_sys::JSON::stringify(&get(&reference, "messages")).unwrap(),
                "{payload}, {mode}"
            );
            assert_eq!(
                Some(caught),
                get(&reference, "caught").as_bool(),
                "{payload}, {mode}"
            );
        }
    }
}
