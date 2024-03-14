//! The Web Components standard creates a browser feature that allows you to create reusable components, called Custom Elements.
//!
//! While web_sys exposes the browser’s CustomElementRegistry interface, it can be hard to use. Creating a Custom Element requires calling customElements.define() and passing it an ES2015 class that extends HTMLElement, which is not currently possible to do directly from Rust.
//!
//! This crate provides a [CustomElement][CustomElement] trait that, when implemented, allows you to encapsulate any Rust structure as a reusable web component without writing any JavaScript. In theory it should be usable with any Rust front-end framework.
//! ```rust
//! impl CustomElement for MyWebComponent {
//!   fn inject_children(&mut self, this: &HtmlElement) {
//!       inject_style(&this, "p { color: green; }");
//!       let node = self.view();
//!       this.append_child(&node).unwrap_throw();
//!   }
//!
//!   fn observed_attributes() -> &'static [&'static str] {
//!       &["name"]
//!   }
//!
//!   fn attribute_changed_callback(
//!       &mut self,
//!       _this: &HtmlElement,
//!       name: String,
//!       _old_value: Option<String>,
//!       new_value: Option<String>,
//!   ) {
//!       if name == "name" {
//!           /* do something... */
//!       }
//!   }
//!
//!   fn connected_callback(&mut self, _this: &HtmlElement) {
//!       log("connected");
//!   }
//!
//!   fn disconnected_callback(&mut self, _this: &HtmlElement) {
//!       log("disconnected");
//!   }
//!
//!   fn adopted_callback(&mut self, _this: &HtmlElement) {
//!       log("adopted");
//!   }
//! }
//!
//! #[wasm_bindgen]
//! pub fn define_custom_elements() {
//!     MyWebComponent::define("my-component");
//! }
//! ```

use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{window, HtmlElement};

/// A custom DOM element that can be reused via the Web Components/Custom Elements standard.
///
/// Note that your component should implement [Default][std::default::Default], which allows the
/// browser to initialize a “default” blank component when a new custom element node is created.
pub trait CustomElement: Default + 'static {
    /// Appends children to the root element, either to the shadow root in shadow mode or to the custom element itself.
    /// Per the [Web Components spec](https://html.spec.whatwg.org/multipage/custom-elements.html#custom-element-conformance),
    /// this is deferred to the first invocation of `connectedCallback()`.
    /// It will run before [connected_callback](CustomElement::connected_callback).
    fn inject_children(&mut self, this: &HtmlElement) {}

    fn render(&mut self, this: &HtmlElement);

    /// Whether a [Shadow root](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_shadow_DOM)
    /// should be attached to the element or not. Shadow DOM encapsulates styles, but makes some DOM manipulation more difficult.
    ///
    /// Defaults to `false`.
    fn shadow() -> bool {
        false
    }

    /// The names of the attributes whose changes should be observed. If an attribute name is in this list,
    /// [attribute_changed_callback](CustomElement::attribute_changed_callback) will be invoked when it changes.
    /// If it is not, nothing will happen when the DOM attribute changes.
    fn observed_attributes() -> &'static [&'static str] {
        &[]
    }

    /// Invoked when the custom element is instantiated. This can be used to inject any code into the `constructor`,
    /// immediately after it calls `super()`.
    fn constructor(&mut self, _this: &HtmlElement) {}

    /// Invoked each time the custom element is appended into a document-connected element.
    /// This will happen each time the node is moved, and may happen before the element's contents have been fully parsed.
    fn connected_callback(&mut self, _this: &HtmlElement) {}

    /// Invoked each time the custom element is disconnected from the document's DOM.
    fn disconnected_callback(&mut self, _this: &HtmlElement) {}

    /// Invoked each time the custom element is moved to a new document.
    fn adopted_callback(&mut self, _this: &HtmlElement) {}

    /// Invoked each time one of the custom element's attributes is added, removed, or changed.
    /// To observe an attribute, include it in [observed_attributes](CustomElement::observed_attributes).
    fn attribute_changed_callback(
        &mut self,
        _this: &HtmlElement,
        _name: String,
        _old_value: Option<String>,
        _new_value: Option<String>,
    ) {
    }

    /// Specifies the built-in element your element inherits from, if any, by giving its tag name and constructor.
    /// This is only relevant to customized built-in elements, not autonomous custom elements.
    /// [Browser support is inconsistent](https://caniuse.com/custom-elementsv1).
    ///
    /// Defaults to the equivalent of `extends HTMLElement`, which makes for an autonomous custom element.
    ///
    /// To specify your own superclass, import it using `wasm_bindgen`:
    /// ```
    /// #[wasm_bindgen]
    /// extern "C" {
    ///     #[wasm_bindgen(js_name = HTMLParagraphElement, js_namespace = window)]
    ///     pub static HtmlParagraphElementConstructor: js_sys::Function;
    /// }
    /// impl CustomElement for MyComponent {
    ///     fn superclass() -> (Option<&'static str>, &'static js_sys::Function) {
    ///         (Some("p"), &HtmlParagraphElementConstructor)
    ///     }
    /// }
    /// ```
    fn superclass() -> (Option<&'static str>, &'static js_sys::Function) {
        (None, &HtmlElementConstructor)
    }

    /// Must be called somewhere to define the custom element and register it with the DOM Custom Elements Registry.
    ///
    /// Note that custom element names must contain a hyphen.
    ///
    /// ```rust
    /// impl CustomElement for MyCustomElement { /* ... */  */}
    /// #[wasm_bindgen]
    /// pub fn define_elements() {
    ///     MyCustomElement::define("my-component");
    /// }
    /// ```
    fn define(tag_name: &'static str) {
        // constructor function will be called for each new instance of the component
        let constructor = Closure::wrap(Box::new(move |this: HtmlElement| {
            let component = Arc::new(Mutex::new(Self::default()));

            // constructor
            let cmp = component.clone();
            let constructor = Closure::wrap(Box::new({
                move |el| {
                    let mut lock = cmp.lock().unwrap_throw();
                    lock.constructor(&el);
                }
            }) as Box<dyn FnMut(HtmlElement)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_constructor"),
                &constructor.into_js_value(),
            )
            .unwrap_throw();

            // inject_children
            let cmp = component.clone();
            let inject_children = Closure::wrap(Box::new({
                move |el| {
                    let mut lock = cmp.lock().unwrap_throw();
                    lock.inject_children(&el);
                }
            }) as Box<dyn FnMut(HtmlElement)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_injectChildren"),
                &inject_children.into_js_value(),
            )
            .unwrap_throw();

            // connectedCallback
            let cmp = component.clone();
            let connected = Closure::wrap(Box::new({
                move |el| {
                    let mut lock = cmp.lock().unwrap_throw();
                    lock.connected_callback(&el);
                }
            }) as Box<dyn FnMut(HtmlElement)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_connectedCallback"),
                &connected.into_js_value(),
            )
            .unwrap_throw();

            // disconnectedCallback
            let cmp = component.clone();
            let disconnected = Closure::wrap(Box::new(move |el| {
                let mut lock = cmp.lock().unwrap_throw();
                lock.disconnected_callback(&el);
            }) as Box<dyn FnMut(HtmlElement)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_disconnectedCallback"),
                &disconnected.into_js_value(),
            )
            .unwrap_throw();

            // adoptedCallback
            let cmp = component.clone();
            let adopted = Closure::wrap(Box::new(move |el| {
                let mut lock = cmp.lock().unwrap_throw();
                lock.adopted_callback(&el);
            }) as Box<dyn FnMut(HtmlElement)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_adoptedCallback"),
                &adopted.into_js_value(),
            )
            .unwrap_throw();

            // attributeChangedCallback
            let cmp = component;
            let attribute_changed = Closure::wrap(Box::new(move |el, name, old_value, new_value| {
                let mut lock = cmp.lock().unwrap_throw();
                lock.attribute_changed_callback(&el, name, old_value, new_value);
            })
                as Box<dyn FnMut(HtmlElement, String, Option<String>, Option<String>)>);
            js_sys::Reflect::set(
                &this,
                &JsValue::from_str("_attributeChangedCallback"),
                &attribute_changed.into_js_value(),
            )
            .unwrap_throw();
        }) as Box<dyn FnMut(HtmlElement)>);

        // observedAttributes is static and needs to be known when the class is defined
        let attributes = Self::observed_attributes();
        let observed_attributes = JsValue::from(
            attributes
                .iter()
                .map(|attr| JsValue::from_str(attr))
                .collect::<js_sys::Array>(),
        );

        // call out to JS to define the Custom Element
        let (super_tag, super_constructor) = Self::superclass();
        make_custom_element(
            super_constructor,
            tag_name,
            Self::shadow(),
            constructor.into_js_value(),
            observed_attributes,
            super_tag,
        );
    }
}

/// Attaches a `<style>` element with the given content to the element,
/// either to its shadow root (if it exists) or to the custom element itself.
///
/// This is an optional helper function; if you use it, you probably want it somewhere
/// in your [inject_children](CustomElement::inject_children) function.
pub fn inject_style(this: &HtmlElement, style: &str) {
    let style_el = window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .create_element("style")
        .unwrap_throw();
    style_el.set_inner_html(style);
    match this.shadow_root() {
        Some(shadow_root) => shadow_root.append_child(&style_el).unwrap_throw(),
        None => this.append_child(&style_el).unwrap_throw(),
    };
}

/// Attaches a `<link rel="stylesheet">` element with the given URL to the custom element,
/// either to its shadow root (if it exists) or to the custom element itself.
///
/// This is an optional helper function; if you use it, you probably want it somewhere
/// in your [inject_children](CustomElement::inject_children) function.
pub fn inject_stylesheet(this: &HtmlElement, url: &str) {
    let style_el = window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .create_element("link")
        .unwrap_throw();
    style_el.set_attribute("rel", "stylesheet").unwrap_throw();
    style_el.set_attribute("href", url).unwrap_throw();
    match this.shadow_root() {
        Some(shadow_root) => shadow_root.append_child(&style_el).unwrap_throw(),
        None => this.append_child(&style_el).unwrap_throw(),
    };
}

// JavaScript shim
#[wasm_bindgen(module = "/src/make_custom_element.js")]
extern "C" {
    fn make_custom_element(
        superclass: &js_sys::Function,
        tag_name: &str,
        shadow: bool,
        constructor: JsValue,
        observed_attributes: JsValue,
        superclass_tag: Option<&str>,
    );
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = HTMLElement, js_namespace = window)]
    pub static HtmlElementConstructor: js_sys::Function;
}
