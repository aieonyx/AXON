// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Computation graph -- sovereign AI forward pass.
// Clean-room: studied computation graph theory from academic papers only.
// P63.0: DAG forward pass, node execution, value propagation.
use crate::tensor::Tensor;
use crate::ops::{matmul, relu, sigmoid, tanh, softmax, add_bias, layer_norm};
use crate::error::{AiError, AiResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum NodeOp {
    Input(String),
    MatMul,
    Add,
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
    AddBias,
    LayerNorm { eps: f32 },
    Constant(Tensor),
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id:      String,
    pub op:      NodeOp,
    pub inputs:  Vec<String>,
}

impl GraphNode {
    pub fn input(id: &str, name: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::Input(name.to_string()), inputs: vec![] }
    }
    pub fn constant(id: &str, tensor: Tensor) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::Constant(tensor), inputs: vec![] }
    }
    pub fn matmul(id: &str, a: &str, b: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::MatMul, inputs: vec![a.to_string(), b.to_string()] }
    }
    pub fn relu(id: &str, input: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::ReLU, inputs: vec![input.to_string()] }
    }
    pub fn sigmoid(id: &str, input: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::Sigmoid, inputs: vec![input.to_string()] }
    }
    pub fn softmax(id: &str, input: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::Softmax, inputs: vec![input.to_string()] }
    }
    pub fn add(id: &str, a: &str, b: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::Add, inputs: vec![a.to_string(), b.to_string()] }
    }
    pub fn add_bias(id: &str, input: &str, bias: &str) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::AddBias, inputs: vec![input.to_string(), bias.to_string()] }
    }
    pub fn layer_norm(id: &str, input: &str, eps: f32) -> Self {
        GraphNode { id: id.to_string(), op: NodeOp::LayerNorm { eps }, inputs: vec![input.to_string()] }
    }
}

pub struct ComputeGraph {
    nodes:  Vec<GraphNode>,
    order:  Vec<String>,
}

impl ComputeGraph {
    pub fn new() -> Self { ComputeGraph { nodes: vec![], order: vec![] } }

    pub fn add_node(&mut self, node: GraphNode) {
        self.order.push(node.id.clone());
        self.nodes.push(node);
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    pub fn forward(
        &self,
        inputs: &HashMap<String, Tensor>,
    ) -> AiResult<HashMap<String, Tensor>> {
        let mut values: HashMap<String, Tensor> = HashMap::new();

        for node_id in &self.order {
            let node = self.nodes.iter().find(|n| &n.id == node_id)
                .ok_or_else(|| AiError::NodeNotFound(node_id.clone()))?;

            let result = match &node.op {
                NodeOp::Input(name) => {
                    inputs.get(name)
                        .or_else(|| inputs.get(node_id.as_str()))
                        .ok_or_else(|| AiError::NodeNotFound(name.clone()))?
                        .clone()
                }
                NodeOp::Constant(t) => t.clone(),
                NodeOp::MatMul => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    let b = self.get_value(&values, inputs, &node.inputs[1])?;
                    matmul(&a, &b)?
                }
                NodeOp::Add => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    let b = self.get_value(&values, inputs, &node.inputs[1])?;
                    a.add(&b)?
                }
                NodeOp::ReLU => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    relu(&a)
                }
                NodeOp::Sigmoid => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    sigmoid(&a)
                }
                NodeOp::Tanh => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    tanh(&a)
                }
                NodeOp::Softmax => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    softmax(&a)?
                }
                NodeOp::AddBias => {
                    let a    = self.get_value(&values, inputs, &node.inputs[0])?;
                    let bias = self.get_value(&values, inputs, &node.inputs[1])?;
                    add_bias(&a, &bias)?
                }
                NodeOp::LayerNorm { eps } => {
                    let a = self.get_value(&values, inputs, &node.inputs[0])?;
                    layer_norm(&a, *eps)?
                }
            };
            values.insert(node_id.clone(), result);
        }
        Ok(values)
    }

    fn get_value(
        &self,
        values: &HashMap<String, Tensor>,
        inputs: &HashMap<String, Tensor>,
        id: &str,
    ) -> AiResult<Tensor> {
        values.get(id)
            .or_else(|| inputs.get(id))
            .cloned()
            .ok_or_else(|| AiError::NodeNotFound(id.to_string()))
    }
}
