use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    OnlineClient, PolkadotConfig,
};
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
    let rpc = try_use_context::<LegacyRpcMethods<PolkadotConfig>>().unwrap();
    let api = try_use_context::<OnlineClient<PolkadotConfig>>().unwrap();
    let block_hash = rpc
        .chain_get_block_hash(Some(props.block_number.into()))
        .await
        .unwrap()
        .unwrap();
    let hex = hex::encode(block_hash);

    let block = api.blocks().at(block_hash).await.unwrap();
    let xt_count = block.extrinsics().await.unwrap().len();
    let ev_count = block.events().await.unwrap().len();

    view! {
        label(class="label") {
            span(class="label-text") {"Block number"}
        }
        (props.block_number)
        label(class="label") {
            span(class="label-text") {"Hash"}
        }
        (hex)
        label(class="label") {
            span(class="label-text") {"Extrinsics"}
        }
        (xt_count)
        label(class="label") {
            span(class="label-text") {"Events"}
        }
        (ev_count)
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

#[component]
async fn Content<G: Html>() -> View<G> {
    let block_number = create_signal(0);
    let blocks_state = create_signal(vec![]);
    let url = "wss://rpc.polkadot.io:443";
    let rpc_client = RpcClient::from_url(&url).await.unwrap();
    let rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
    let api = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client)
        .await
        .unwrap();
    provide_context(rpc);
    provide_context(api.clone());

    spawn_local_scoped(async move {
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
}

fn main() {
    sycamore::render(|| {
        view! {Content()}
    });
}
