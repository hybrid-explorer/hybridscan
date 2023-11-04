use subxt::{OnlineClient, PolkadotConfig};
use sycamore::{futures::spawn_local_scoped, prelude::*};
use sycamore_router::{HistoryIntegration, Route, Router, RouterProps};

#[derive(Route, Copy, Clone)]
enum AppRoutes {
    #[to("/")]
    Home,
    #[to("/block/<block_number>")]
    Block { block_number: u32 },
    #[not_found]
    NotFound,
}

#[derive(Props)]
struct BlockProps {
    block_number: u32,
}

#[component]
async fn Block<G: Html>(props: BlockProps) -> View<G> {
    let api = try_use_context::<OnlineClient<PolkadotConfig>>();

    let msg = match api {
        Some(api) => "okay",
        None => "Not okay",
    };

    let test = use_context::<u8>();

    view! {
        p { (props.block_number) }
        p { (msg) }
        p { (test) }
    }
}

#[derive(Props)]
struct BlocksProps {
    blocks: ReadSignal<Vec<u32>>,
}

#[component]
async fn Blocks<G: Html>(props: BlocksProps) -> View<G> {
    view! {
        table(class="table") {
            thead {
                tr {
                    th {"Block"}
                    th {"Age"}
                    th {"Producer"}
                }
            }
            tbody{
                Keyed(
                    iterable=props.blocks,
                    view=|x| view! {
                        tr{td { (x) }}
                    },
                    key=|x| *x,
                )
            }
        }
    }
}

fn main() {
    sycamore::render(|| {
        let block_number = create_signal(0);
        let blocks_state = create_signal(vec![]);

        let test: u8 = 4;
        provide_context(test);

        spawn_local_scoped(async move {
            let url = "wss://rpc.polkadot.io:443";
            let api = OnlineClient::<PolkadotConfig>::from_url(url).await.unwrap();
            provide_context(api.clone());
            let mut blocks_sub = api.blocks().subscribe_finalized().await.unwrap();

            while let Some(block) = blocks_sub.next().await {
                let block = block.unwrap();
                block_number.set(block.number());
                let mut blocks = blocks_state.take();
                blocks.insert(0, block.number());
                blocks_state.set(blocks);
            }
        });
        view! {
            div(class="w-full navbar bg-base-300") {
                div(class="flex-1 px-2 mx-2") { "HybridScan" }
                p { (block_number.get()) }
            }
            div(class="p-8") {
                Router(
                    integration=HistoryIntegration::new(),
                    view=move |route: ReadSignal<AppRoutes>| {
                        view! {
                            div(class="app") {
                                (match route.get() {
                                    AppRoutes::Home => Blocks(BlocksProps { blocks: *blocks_state}),
                                    AppRoutes::Block{block_number} => Block(BlockProps{block_number:block_number}),
                                    AppRoutes::NotFound => view! {
                                        "404 Not Found"
                                    },
                                })
                            }
                        }
                    }
                )
            }
        }
    });
}
