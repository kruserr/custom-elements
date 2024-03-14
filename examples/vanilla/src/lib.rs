use custom_elements::CustomElement;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlElement, Node, Text};

struct MyWebComponent {
    name: String,
}

impl MyWebComponent {
    fn new() -> Self {
        Self { name: "".into() }
    }
}

impl Default for MyWebComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomElement for MyWebComponent {
    fn observed_attributes() -> &'static [&'static str] {
        &["name"]
    }

    fn attribute_changed_callback(
        &mut self,
        _this: &HtmlElement,
        name: String,
        _old_value: Option<String>,
        new_value: Option<String>,
    ) {
      if new_value == _old_value {
        return;
      }

      if name == "name" {
        self.name = new_value.unwrap_or("".to_owned());
      }

      self.render(_this);
    }

    fn connected_callback(&mut self, _this: &HtmlElement) {
      self.render(_this);
    }
    
    fn render(&mut self, _this: &HtmlElement) {
      let name = &self.name;
      _this.set_inner_html(&format!(r#"
        <div style="display: inline-block; background: #262626; padding: 8px; border-radius: 8px;">
          {name}
        </div>
      "#));
    }
}

// wasm_bindgen entry point defines the Custom Element, then creates a few of them
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    // define the Custom Element
    MyWebComponent::define("ce-vanilla");

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
