use gloo_console::log;
use gloo_net::http::Request;
use serde_wasm_bindgen;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{html, Callback, Event, Html};

mod command;

#[function_component]
fn App() -> Html {
    let api_url = std::env!("API_URL");
    let query_handle = use_state(|| String::default());
    let query = (*query_handle).clone();
    let input_node_ref = use_node_ref();

    let onchange: Callback<Event> = {
        let input_node_ref = input_node_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = input_node_ref.cast::<HtmlInputElement>() {
                query_handle.set(input.value())
            }
        })
    };

    let results_handle = use_state(|| Vec::new());
    let results = (*results_handle).clone();

    let onclick: Callback<MouseEvent> = {
        Callback::from({
            let query = query.clone();
            let results = results_handle.clone();
            move |_| {
                let query = query.clone();
                let results = results.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let res = Request::get(&format!("{}/search?query={}", api_url, query))
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    let res: Vec<command::SubCommand> = serde_json::from_value(res).unwrap();

                    log!(serde_wasm_bindgen::to_value(&res).unwrap());

                    results.set(res);
                });
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
                        {onchange}
                        ref={input_node_ref}
                        value={query}
                    />
                    <button
                        class="rounded-full bg-[#301F18] p-4 text-[#FF5B04] hover:opacity-80"
                        {onclick}
                    >
                        {"Search"}
                    </button>
                </div>
                {
                    if results.len() > 0 {
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
