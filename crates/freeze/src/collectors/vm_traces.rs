use crate::{
    dataframes::SortableDataFrame, store, with_series, CollectByBlock, CollectByTransaction,
    CollectError, ColumnData, Datatype, Params, Schemas, Source, Table, ToVecU8, VmTraces,
};
use ethers::prelude::*;
use polars::prelude::*;
use std::collections::HashMap;

type Result<T> = ::core::result::Result<T, CollectError>;

#[async_trait::async_trait]
impl CollectByBlock for VmTraces {
    type Response = (Option<u32>, Option<Vec<u8>>, Vec<ethers::types::BlockTrace>);

    type Columns = VmTraceColumns;

    async fn extract(request: Params, source: Source, _schemas: Schemas) -> Result<Self::Response> {
        source.fetcher.trace_block_vm_traces(request.block_number() as u32).await
    }

    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        process_vm_traces(response, columns, schemas)
    }
}

#[async_trait::async_trait]
impl CollectByTransaction for VmTraces {
    type Response = (Option<u32>, Option<Vec<u8>>, Vec<ethers::types::BlockTrace>);

    type Columns = VmTraceColumns;

    async fn extract(request: Params, source: Source, _schemas: Schemas) -> Result<Self::Response> {
        source.fetcher.trace_transaction_vm_traces(request.transaction_hash()).await
    }

    fn transform(response: Self::Response, columns: &mut Self::Columns, schemas: &Schemas) {
        process_vm_traces(response, columns, schemas)
    }
}

/// columns for transactions
#[cryo_to_df::to_df(Datatype::VmTraces)]
#[derive(Default)]
pub struct VmTraceColumns {
    block_number: Vec<Option<u32>>,
    transaction_hash: Vec<Option<Vec<u8>>>,
    transaction_position: Vec<u32>,
    pc: Vec<u64>,
    cost: Vec<u64>,
    used: Vec<Option<u64>>,
    push: Vec<Option<Vec<u8>>>,
    mem_off: Vec<Option<u32>>,
    mem_data: Vec<Option<Vec<u8>>>,
    storage_key: Vec<Option<Vec<u8>>>,
    storage_val: Vec<Option<Vec<u8>>>,
    op: Vec<String>,
    n_rows: usize,
}

fn process_vm_traces(
    response: (Option<u32>, Option<Vec<u8>>, Vec<ethers::types::BlockTrace>),
    columns: &mut VmTraceColumns,
    schemas: &HashMap<Datatype, Table>,
) {
    let (block_number, tx, block_traces) = response;
    let schema = schemas.get(&Datatype::BalanceDiffs).expect("missing schema");
    for (tx_pos, block_trace) in block_traces.into_iter().enumerate() {
        if let Some(vm_trace) = block_trace.vm_trace {
            add_ops(vm_trace, schema, columns, block_number, tx.clone(), tx_pos);
        }
    }
}

fn add_ops(
    vm_trace: VMTrace,
    schema: &Table,
    columns: &mut VmTraceColumns,
    number: Option<u32>,
    tx_hash: Option<Vec<u8>>,
    tx_pos: usize,
) {
    for opcode in vm_trace.ops {
        columns.n_rows += 1;

        store!(schema, columns, block_number, number);
        store!(schema, columns, transaction_hash, tx_hash.clone());
        store!(schema, columns, transaction_position, tx_pos as u32);
        store!(schema, columns, pc, opcode.pc as u64);
        store!(schema, columns, cost, opcode.cost);
        if let Some(ex) = opcode.ex {
            store!(schema, columns, used, Some(ex.used));
            store!(schema, columns, push, Some(ex.push.to_vec_u8()));

            if let Some(mem) = ex.mem {
                store!(schema, columns, mem_off, Some(mem.off as u32));
                store!(schema, columns, mem_data, Some(mem.data.to_vec()));
            } else {
                store!(schema, columns, mem_off, None);
                store!(schema, columns, mem_data, None);
            };
            if let Some(store) = ex.store {
                store!(schema, columns, storage_key, Some(store.key.to_vec_u8()));
                store!(schema, columns, storage_val, Some(store.val.to_vec_u8()));
            } else {
                store!(schema, columns, storage_key, None);
                store!(schema, columns, storage_val, None);
            }
        } else {
            store!(schema, columns, used, None);
            store!(schema, columns, push, None);
            store!(schema, columns, mem_off, None);
            store!(schema, columns, mem_data, None);
            store!(schema, columns, storage_key, None);
            store!(schema, columns, storage_val, None);
        }
        if schema.has_column("op") {
            match opcode.op {
                ExecutedInstruction::Known(op) => store!(schema, columns, op, op.to_string()),
                ExecutedInstruction::Unknown(op) => store!(schema, columns, op, op),
            }
        };

        if let Some(sub) = opcode.sub {
            add_ops(sub, schema, columns, number, tx_hash.clone(), tx_pos)
        }
    }
}
