#![cfg(target_arch = "wasm32")]
//! Synchronous Attach listener reentry, including a real WebSocket event target.
use js_sys::{Function, Object};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::AttachAddon};
wasm_bindgen_test_configure!(run_in_browser);

async fn run(source: &str) {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = AttachAddon::new(Object::new().unchecked_into(), JsValue::UNDEFINED)
        .unwrap()
        .into();
    let promise = Function::new_with_args("Terminal,AttachAddon", source)
        .call2(
            &JsValue::UNDEFINED,
            &js_sys::Reflect::get(&terminal, &"constructor".into()).unwrap(),
            &js_sys::Reflect::get(&addon, &"constructor".into()).unwrap(),
        )
        .unwrap();
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await;
    let free = Function::new_with_args("value", "value.free();");
    free.call1(&JsValue::UNDEFINED, &addon).unwrap();
    free.call1(&JsValue::UNDEFINED, &terminal).unwrap();
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn attach_callbacks_match_unchanged_source_reentry_and_disposal() {
    let source = format!(
        r#"
{}
return (async()=>{{
  const actual=await attachReentryCases(socket=>{{
    const term=new Terminal();
    const addon=new AttachAddon(socket,{{bidirectional:false}});
    addon.activate(term);
    return {{
      dispose:()=>addon.dispose(),
      flush:async()=>{{
        await new Promise(resolve=>term.write('',resolve));
        return term.buffer.active.getLine(0).translateToString(true);
      }},
      free:()=>{{term.dispose();addon.free();term.free();}}
    }};
  }});
  const expected={};
  if(JSON.stringify(actual)!==JSON.stringify(expected)){{
    throw Error('Attach source mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
  }}
}})();
"#,
        include_str!("fixtures/attach-reentry.js"),
        include_str!("fixtures/attach-reentry.json")
    );
    run(&source).await;
}

#[wasm_bindgen_test]
async fn real_socket_nested_message_keeps_order_and_survives_disposal() {
    let Some(url) = option_env!("UTILITY_SOCKET_URL") else {
        panic!("run with scripts/utility-addon-browser-check.mjs");
    };
    let source = format!(
        r#"
return (async()=>{{
  const term=new Terminal();
  const socket=new WebSocket({url:?});
  const addon=new AttachAddon(socket,{{bidirectional:false}});
  const errors=[];
  const capture=event=>{{errors.push(String(event.error));event.preventDefault();}};
  const opened=new Promise((resolve,reject)=>{{
    socket.addEventListener('open',resolve,{{once:true}});
    socket.addEventListener('error',reject,{{once:true}});
  }});
  addEventListener('error',capture);
  try {{
    await opened;
    // The peer's own messages are irrelevant to this synchronous event trace.
    // A real WebSocket is also an EventTarget, so use it without replacing APIs.
    addon.activate(term);
    const outer=new MessageEvent('message');
    Object.defineProperty(outer,'data',{{get(){{
      socket.dispatchEvent(new MessageEvent('message',{{data:'B'}}));
      return 'A';
    }}}});
    socket.dispatchEvent(outer);
    addon.dispose();
    socket.dispatchEvent(new MessageEvent('message',{{data:'ignored'}}));
    await new Promise(resolve=>term.write('',resolve));
    const text=term.buffer.active.getLine(0).translateToString(true);
    if(text!=='BA'||errors.length)throw Error('real socket trace '+JSON.stringify({{text,errors}}));
  }} finally {{
    removeEventListener('error',capture);
    addon.dispose();term.dispose();addon.free();term.free();
    const closed=new Promise(resolve=>socket.addEventListener('close',resolve,{{once:true}}));
    if(socket.readyState!==WebSocket.CLOSED){{socket.close();await closed;}}
  }}
}})();
"#
    );
    run(&source).await;
}
