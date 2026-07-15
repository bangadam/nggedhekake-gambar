use dioxus::prelude::*;

#[component]
pub fn ImageSlider(before: String, after: String) -> Element {
    let mut position = use_signal(|| 50.0_f32);
    let reveal = position().clamp(0.0, 100.0);
    let clip = format!("inset(0 {}% 0 0)", 100.0 - reveal);

    rsx! {
        div { class: "comparison",
            img { class: "comparison-image after-base", src: "{after}", alt: "Upscaled result" }
            img {
                class: "comparison-image before-overlay",
                src: "{before}",
                alt: "Original image",
                style: "clip-path: {clip}"
            }
            div { class: "comparison-divider", style: "left: {reveal}%",
                span { class: "divider-handle", "‹  ›" }
            }
            span { class: "comparison-label before-label", "BEFORE" }
            span { class: "comparison-label after-label", "AFTER" }
            input {
                class: "comparison-range",
                r#type: "range",
                min: "0",
                max: "100",
                value: "{reveal}",
                aria_label: "Reveal before and after image",
                oninput: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        position.set(value);
                    }
                }
            }
        }
    }
}
