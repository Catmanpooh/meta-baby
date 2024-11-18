use crate::error_template::{AppError, ErrorTemplate};
use ev::SubmitEvent;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use validator::Validate;
use serde::{Serialize, Deserialize};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/meta-baby.css"/>
        <Stylesheet  id="google_font" href="https://fonts.googleapis.com/css?family=Press+Start+2P"/>
        <Stylesheet  href="https://unpkg.com/nes.css/css/nes.css"/>
        
        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {

    view! {
        <h1>"Welcome to Leptos!"</h1>
       <div class=".container">
       <Qrform/>
       </div>
    }
}

#[derive(Debug, Default, Validate, Deserialize, Serialize, Clone)]
pub struct QRInfoType {
    pub contents: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>
}


#[component]
fn Qrform() -> impl IntoView {
    let qr_info = create_server_action::<QrCodePost>();

    let response = qr_info.value();
    

    let initial_length = 1;
    let mut next_counter_id = initial_length;

    let initial_counters = (0..initial_length)
        .map(|id| (id, create_signal(id + 1)))
        .collect::<Vec<_>>();

    let (counters, set_counters) = create_signal(initial_counters);
    
  
    let add_counter = move |_| {
        let sig = create_signal(next_counter_id + 1);
        set_counters.update(move |counters| {
            counters.push((next_counter_id, sig))
        });
        next_counter_id += 1;
    };

    let on_submit = move |ev: SubmitEvent| {

        let data = QRInfoType::from_event(&ev);
       
        leptos::logging::log!("Submitting data: {:?}", data);
        if data.is_err() {
            ev.prevent_default();
        }
    };

    view! {
        <div class=".form-container" style="display: flex; gap: 4em;">
           
            <ActionForm 
                action=qr_info
                on:submit=on_submit
                class="form"
            >
            <button class="nes-btn is-primary" on:click=add_counter>
            "Add More QR Information"
            </button>
                <For
                    each=counters
                    key=|counter| counter.0
                    children=move |(id, _)| {
                        let id_clone = id.clone();
                        view! {
                            <div>
                                <div class="nes-field">
                                    <label for=format!("content_field_{}", id)>"Qr Information"</label>
                                    <input 
                                        type="text" 
                                        name="contents[]"
                                        id=format!("content_field_{}", id)
                                        class="nes-input"
                                    />
                                </div>
                                <button
                                    type="button"
                                    class="nes-btn is-error"
                                    on:click=move |_| {
                                        set_counters.update(|counters| {
                                            counters.retain(|(counter_id, _)| counter_id != &id_clone)
                                        });
                                    }
                                >
                                    "Remove"
                                </button>
                            </div>
                        }
                    }
                />

                <div class="nes-field">
                    <label for="height">"Qr Height"</label>
                    <input 
                        type="number" 
                        name="height"
                        id="height"
                        class="nes-input"
                    />
                </div>

                <div class="nes-field">
                    <label for="width">"Qr Width"</label>
                    <input 
                        type="number" 
                        name="width"
                        id="width"
                        class="nes-input"
                    />
                </div>
                <br/>
                <button class="nes-btn is-success" type="submit">
                    "Submit"
                </button>
            </ActionForm>
            <div class=".image-container">
            
            {move || response.get().map(move |data| match data {
                Ok(p) => {
                    view! {
                        <img src={p} type="image/png" alt="QR image" />
                    }.into_view()
                },
                Err(e) => view! {
                    <div>
                        <h3>"Oh nooooo we had an issue! Try again"</h3>
                    </div>
                }.into_view()
            } )}
            </div>
        </div>
    }
}


#[component]
fn QrPage() -> impl IntoView {
    let get_qr_image = create_resource(|| (), |_| async move { qr_code_get().await });

    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p>"Loading data..."</p> }
            }>
                {
                    move || {
                        get_qr_image.get().map(|data| {
                            match data {
                                Ok(qr_image) => {
                                    view! {
                                        <img src={qr_image} type="image/png" alt="QR image" />
                                    }.into_view()
                                },
                                Err(_) => view! {
                                    <div>
                                        <h1>"Oh nooooo we had an issue! Try again"</h1>
                                    </div>
                                }.into_view()
                            }
                        })
                    }
                }
            </Suspense>
        </div>
    }
}

fn vec_to_data_url(image_data: Vec<u8>, mime_type: &str) -> String {
    let base64 = STANDARD.encode(image_data);
    format!("data:{};base64,{}", mime_type, base64)
}

#[server(QrCode, "/api")]
pub async fn qr_code_get() -> Result<String, ServerFnError> {

    use fast_qr::{
        convert::{image::ImageBuilder, Builder, Shape},
        QRBuilder, Version, ECL,
    };
   
    let qrcode = QRBuilder::new("https://example12374547.com/")
        .ecl(ECL::H)
        .version(Version::V10)
        .build()?;

    let image_as_bytes = ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .fit_width(512)
        .background_color([255, 255, 255, 255]) // opaque
        .to_bytes(&qrcode);
    
    // leptos::logging::log!("image_as_bytes trip back: {:?}", image_as_bytes);
    
    match image_as_bytes {
        Ok(image) => Ok(vec_to_data_url(image, "image/png")),
        Err(e) => {
            leptos::logging::log!("Error : {:?}", e);
            Err(ServerFnError::Args(String::from(
                "Error with the title or post!",
        )))}, 
    }
}

#[server(QrCodePost, "/api")]
pub async fn qr_code_post(
    contents: Vec<String>,
    width: Option<u32>,
    height: Option<u32>
) -> Result<String, ServerFnError> {

    use fast_qr::{
        convert::{image::ImageBuilder, Builder, Shape},
        QRBuilder, Version, ECL,
    };

    leptos::logging::log!("image_info trip back: {:?}",contents );
    let qr_info:Vec<u8> =  contents.into_iter()
    .flat_map(|s| s.into_bytes()) // Convert each String into bytes and flatten
    .collect();

    let qrcode = QRBuilder::new(qr_info)
        .ecl(ECL::H)
        .version(Version::V10)
        .build()?;

    let image_as_bytes = ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .fit_height(height.unwrap_or(512))
        .fit_width(width.unwrap_or(512))
        .background_color([255, 255, 255, 255]) // opaque
        .to_bytes(&qrcode);
    
    
    match image_as_bytes {
        Ok(image) => Ok(vec_to_data_url(image, "image/png")),
        Err(e) => {
            leptos::logging::log!("Error : {:?}", e);
            Err(ServerFnError::Args(String::from(
                "Error with the title or post!",
        )))}, 
    }
}