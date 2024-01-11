use crate::web_sys_ext::window;
use gloo_console::log;
use gloo_net::http::Request;
use serde_wasm_bindgen;
use web_sys::HtmlInputElement;
use yew::prelude::*;
mod web_sys_ext;
use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use std::rc::Rc;
use yew::{html, Callback, Html};
mod command;

#[function_component]
fn ClipBoardIcon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
        </svg>
    }
}

#[function_component]
fn App() -> Html {
    let api_url = std::env!("API_URL");
    let input_node_ref = use_node_ref();
    let timeout_handle = use_state(|| Rc::new(None::<Timeout>));
    let show_copied_notification_handle = use_state(|| false);
    let show_copied_notification = (*show_copied_notification_handle).clone();
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

    let clipboard: Callback<String> = {
        Callback::from({
            move |input: String| {
                let input = input.clone();
                let show_copied_notification_handle = show_copied_notification_handle.clone();
                let window = window().expect("should have a Window");
                let clipboard = window
                    .navigator()
                    .clipboard()
                    .expect("should have Clipboard");
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = clipboard.write_text(&input);
                    show_copied_notification_handle.set(true);
                    TimeoutFuture::new(1000).await;
                    show_copied_notification_handle.set(false);
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
                let input_node_ref = input_node_ref.clone();
                let search = search.clone();

                let timeout_handle = timeout_handle.clone();
                let timeout = Rc::clone(&timeout_handle);
                if e.key() == "Enter" {
                    if let Some(input) = input_node_ref.cast::<HtmlInputElement>() {
                        search.emit(input.value());
                    }
                } else {
                    if let Some(input) = input_node_ref.cast::<HtmlInputElement>() {
                        wasm_bindgen_futures::spawn_local(async move {
                            let input = input.clone();
                            if let Ok(timeout) = Rc::try_unwrap(timeout) {
                                timeout.unwrap().cancel();
                            }
                            let timeout = Timeout::new(500, move || {
                                if input.value().len() > 0 {
                                    search.emit(input.value());
                                }
                            });
                            timeout_handle.set(Rc::new(Some(timeout)));
                        });
                    }
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
                        <div class="bg-[#1A1A1A] flex flex-col space-y-2 rounded-md p-4 relative">
                            {
                                    results.into_iter().map(|result| {
                                        let command = result.command.clone();
                                        let clipboard = clipboard.clone();
                                        html! {
                                            <div class="bg-[#252525] rounded-md text-xl w-full  rounded-md text-white flex space-y-2 p-4 items-center justify-between" >
                                                <div class="flex flex-col space-y-2">
                                                    <h1 class="text-lg text-white"> {result.command} </h1>
                                                    <p class="text-gray-400 text-sm"> {result.description} </p>
                                                </div>
                                                <button
                                                onclick={move |_| clipboard.emit(command.clone())}
                                                class="bg-gray-100/10 p-3 rounded-md hover:bg-gray-100/20">
                                                    <ClipBoardIcon />
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Html>()

                            }
                        {
                            if show_copied_notification {
                                html! {
                                    <div class="bg-white rounded-full text-lg px-4 text-black flex text-center py-2 absolute bottom-5 left-[40%]" >
                                        <p>{"Copied to Clipboard!"}</p>
                                    </div>
                                }
                            } else {
                                html! {<> </>}
                            }
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
