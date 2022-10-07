mod utils;
pub use utils::heap;

mod runner;
pub use runner::Runner;

mod stack;
pub use stack::Stack;

mod http;
pub use http::{execute_node, send_http_request};

mod errors;
pub use errors::RequestError;

use gluescript::GlueNode;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

type MuxNode = Arc<Mutex<GlueNode>>;
type HeapMap = LockedMap<String, String>;
type DepMap = LockedMap<u32, String>;
type LockedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type ParallelExecutionLayer = Vec<MuxNode>;
type ExecutionStack = Vec<ParallelExecutionLayer>;
