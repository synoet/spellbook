use gloo_console::log;
use gloo_net::http::Request;
use serde_wasm_bindgen;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{html, Callback, Html};
mod command;

#[function_component]
fn App() -> Html {
    let api_url = std::env!("API_URL");
    let input_node_ref = use_node_ref();

    let results_handle = use_state(|| Vec::new());
    let results = (*results_handle).clone();
    let is_loading_handle = use_state(|| false);
    let is_loading = (*is_loading_handle).clone();

    let search: Callback<String> = {
        Callback::from({
            let is_loading = is_loading_handle.clone();
            move |query| {
                let results_handle = results_handle.clone();

                let is_loading = is_loading.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    is_loading.set(true);
                    let res = Request::get(&format!("{}/search?query={}", api_url, query))
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    let res: Vec<command::SubCommand> = serde_json::from_value(res).unwrap();

                    log!(serde_wasm_bindgen::to_value(&res).unwrap());

                    results_handle.set(res);

                    is_loading.set(false);
                });
            }
        })
    };

    let onclick: Callback<MouseEvent> = {
        Callback::from({
            let search = search.clone();
            let input_node_ref = input_node_ref.clone();

            move |_| {
                if let Some(input) = input_node_ref.cast::<HtmlInputElement>() {
                    search.emit(input.value());
                }
            }
        })
    };

    let onkeydown: Callback<KeyboardEvent> = {
        Callback::from({
            let search = search.clone();
            let input_node_ref = input_node_ref.clone();

            move |e: KeyboardEvent| {
                if e.key() == "Enter" {
                    if let Some(input) = input_node_ref.cast::<HtmlInputElement>() {
                        search.emit(input.value());
                    }
                } else {
                    ()
                }
            }
        })
    };

    html! {
        <div class="bg-[#131313] w-screen h-screen flex flex-col justify-center items-center text-white">
            <div class="w-[800px] flex flex-col space-y-2">
                <h1 class="text-4xl font-bold"> {"Spellbook"} </h1>
                <p class = "text-md text-gray-400 pb-4"> {"A spellbook for cli commands, search for commands from an embedded registry."} </p>
                <div class="bg-[#1A1A1A] px-8 text-xl w-full h-[100px] rounded-md text-white flex items-center justify-between" >
                    <input
                        class="bg-transparent focus-none border-none outline-none text-white w-full h-full"
                        placeholder="Search for a command"
                        {onkeydown}
                        ref={input_node_ref}
                    />
                    <button
                        class="rounded-full bg-[#301F18] p-4 text-[#FF5B04] hover:opacity-80"
                        {onclick}
                    >
                        {"Search"}
                    </button>
                </div>
                {
                     if is_loading {
                        html! {

                            <div class="bg-[#1A1A1A] flex flex-col space-y-2 rounded-md p-4">
                                {
                                    vec![0,1,2,3,4].into_iter().map(|_| {
                                        html! {
                                            <div class="animate-pulse bg-[#252525] w-[768.1px] h-[87.98px] rounded-md text-xl w-full  rounded-md text-white flex space-y-2 p-4 flex-col" />
                                        }
                                    }).collect::<Html>()
                                }

                            <p class="text-center text-gray-400 pt-4">{"Can't find the command you're looking for? contribute to the registry "} <a class="text-[#FF5B04]" href="https://github.com/synoet/spellbook-registry">{"here"}</a></p>
                            </div>
                        }

                    } else                     if results.len() > 0 {
                        html! {
                        <div class="bg-[#1A1A1A] flex flex-col space-y-2 rounded-md p-4">
                            {
                                    results.into_iter().map(|result| {
                                        html! {
                                            <div class="bg-[#252525] rounded-md text-xl w-full  rounded-md text-white flex space-y-2 p-4 flex-col" >
                                                <h1 class="text-lg text-white"> {result.command} </h1>
                                                <p class="text-gray-400 text-sm"> {result.description} </p>
                                            </div>
                                        }
                                    }).collect::<Html>()

                            }

                            <p class="text-center text-gray-400 pt-4">{"Can't find the command you're looking for? contribute to the registry "} <a class="text-[#FF5B04]" href="https://github.com/synoet/spellbook-registry">{"here"}</a></p>
                        </div>
                        }
                    } else {
                        html!{<div></div>}
                    }
                }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
