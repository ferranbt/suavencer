#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use reth::{
    builder::{components::PayloadServiceBuilder, node::FullNodeTypes, BuilderContext},
    cli::{config::PayloadBuilderConfig, Cli},
    payload::PayloadBuilderHandle,
    providers::CanonStateSubscriptions,
    transaction_pool::TransactionPool,
};
use reth_basic_payload_builder::{BasicPayloadJobGenerator, BasicPayloadJobGeneratorConfig};
use reth_basic_payload_builder::{BuildArguments, BuildOutcome, PayloadBuilder, PayloadConfig};
use reth_node_ethereum::{EthEngineTypes, EthereumNode};
use reth_payload_builder::PayloadBuilderService;

//use alloy_sol_types::{sol, SolCall};
//use Vm::{F11Call, X};

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct CustomPayloadBuilder;

impl<Node, Pool> PayloadServiceBuilder<Node, Pool> for CustomPayloadBuilder
where
    Node: FullNodeTypes<Engine = EthEngineTypes>,
    Pool: TransactionPool + Unpin + 'static,
{
    async fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<PayloadBuilderHandle<Node::Engine>> {
        tracing::info!("Spawning a custom payload builder");
        let conf = ctx.payload_builder_config();
        let payload_builder = reth_optimism_payload_builder::OptimismPayloadBuilder::new(
            ctx.chain_spec(),
            ctx.evm_config().clone(),
        );

        let conf = ctx.payload_builder_config();

        let payload_job_config = BasicPayloadJobGeneratorConfig::default()
            .interval(conf.interval())
            .deadline(conf.deadline())
            .max_payload_tasks(conf.max_payload_tasks())
            // no extradata for OP
            .extradata(Default::default())
            .max_gas_limit(conf.max_gas_limit());

        let payload_generator = BasicPayloadJobGenerator::with_builder(
            ctx.provider().clone(),
            pool,
            ctx.task_executor().clone(),
            payload_job_config,
            ctx.chain_spec(),
            payload_builder,
        );

        let (x, y) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream());

        let (payload_service, payload_builder) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream());

        /*ctx.task_executor()
            .spawn_critical("payload builder service", Box::pin(payload_service));
        */

        Ok(payload_builder)
    }
}

fn main() {
    Cli::parse_args()
        .run(|builder, _| async move {
            let handle = builder
                .with_types(EthereumNode::default())
                // Configure the components of the node
                // use default ethereum components but use our custom payload builder
                .with_components(
                    EthereumNode::components().payload(CustomPayloadBuilder::default()),
                )
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}

/*
sol! {
    #[sol(abi)]
    interface Vm {
        struct X {
            uint64 a;
        }

        function F11(X a) returns (X a);
    }
}

fn x() {
    let x = F11Call { a: X { a: 1 } };
    x.abi_encode();
}
*/

#[cfg(test)]
mod tests {
    //use self::Vm::{F11Call, X};

    use super::*;

    #[test]
    fn larger_can_hold_smaller() {
        //let x = X { a: 1 };
    }
}
