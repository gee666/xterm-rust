# Scrollbar DOM contract

Bharata owns task166 and files `scroll/{abstract_scrollbar,horizontal_scrollbar,vertical_scrollbar,scrollbar_events,scrollbar_layout,pointer_monitor,arrow}.rs`. Lukeni explicitly handed the Arrow portion of task165 to task166; task165 keeps geometry, visibility, FastDomNode and Widget. Shared model is `scroll::browser_scheduler::BrowserScrollable`. Root `scroll/mod.rs` remains Norman-owned.

The oriented classes are thin wrappers around a shared `Scrollbar` and an `Axis` enum. Shared state, rendering and drag behavior are not copied. Constructors are `HorizontalScrollbar::new(model, options, host, is_windows)` and `VerticalScrollbar::new(...)`, returning `Result<Self, JsValue>`. Model handles clone their existing shared state. Options are `Rc<RefCell<ResolvedOptions>>`; each source expression clones needed fields and releases the borrow before JS coercion, DOM calls or callbacks.

`ScrollbarHost` holds `drag_start` and `drag_end` as `Rc<dyn Fn() -> Result<(), JsValue>>`, and a shared `Rc<IframeCache>` for the source Widget click normalization. Source `IScrollbarHost.handleMouseWheel` is never called by these scrollbar classes. Wheel normalization/forwarding remains in ScrollableElement.

Common methods are `dom_node() -> HtmlElement`, `slider_node() -> Result<HtmlElement, JsValue>`, `handle_scroll(&ScrollEvent) -> Result<bool, JsValue>`, `render()`, `begin_reveal()`, `begin_hide()`, `delegate_pointer_down(&Event)`, `update_scrollbar_size(&V)` and `update_options(&Rc<RefCell<ResolvedOptions>>)`. Fallible methods return Result. `is_needed() -> bool` and `write_scroll_position(&mut PositionUpdate, f64)` follow the axis. The explicit Windows platform-service flag preserves the source's strictly-greater-than140 orthogonal drag reset, not a configurable distance policy.

Pointer monitor has cloneable `GlobalPointerMoveMonitor`, `new`, `is_monitoring`, `dispose`, `stop_monitoring(bool) -> Result`, and `start_monitoring(Element, JsValue, JsValue, Rc<dyn Fn(Event) -> Result<(), JsValue>>, Rc<dyn Fn() -> Result<(), JsValue>>) -> Result`. Movement callbacks receive the original DOM event. Capture denial falls back to the element's owner window. Capture release errors are intentionally ignored only because the source does so. `downgrade().upgrade()` supports reentrant callbacks without ownership cycles.

Disposal order is visibility timer, pointer monitor, root pointer listener, vertical arrows in creation order, then slider pointer and click listeners. Pointer disposal does not invoke drag-end or remove the active class, matching source. DOM nodes are not removed by scrollbar disposal. Source delegate hit-testing uses client-rect top for both axes and omits the arrow offset. Preserve those source choices instead of silently fixing them.

Source-only development and controlled build slots remain in effect. No new browser/runtime pass is claimed by this interface document.
