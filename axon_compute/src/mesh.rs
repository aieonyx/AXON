// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute::mesh — AWP Mesh Distributed Compute
// Node registry, task descriptors, mesh dispatcher.
// Interface layer — wires to live AWP stack in deployment.

use alloc::vec::Vec;
use alloc::string::String;

// ------------------------------------------------------------------
// Node identity
// ------------------------------------------------------------------

/// Unique identifier for a mesh node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self { Self(id) }
}

/// Capability profile of a mesh node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCapability {
    /// CPU-only sovereign node (BASTION OS).
    Cpu,
    /// GPU-equipped node (CUDA/ROCm).
    Gpu,
    /// seL4 microkernel node.
    Sel4,
}

/// A registered mesh node.
#[derive(Clone, Debug)]
pub struct MeshNode {
    pub id:         NodeId,
    pub capability: NodeCapability,
    pub label:      String,
    /// True if the node is currently available for dispatch.
    pub available:  bool,
}

impl MeshNode {
    pub fn new(id: u64, capability: NodeCapability, label: &str) -> Self {
        Self {
            id: NodeId(id),
            capability,
            label: alloc::string::ToString::to_string(label),
            available: true,
        }
    }

    pub fn mark_unavailable(&mut self) { self.available = false; }
    pub fn mark_available(&mut self)   { self.available = true;  }
}

// ------------------------------------------------------------------
// Task descriptor — a compute job to dispatch over the mesh
// ------------------------------------------------------------------

/// Priority level for mesh tasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low    = 0,
    Normal = 1,
    High   = 2,
}

/// Status of a dispatched task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Complete,
    Failed,
}

/// A compute task to be dispatched to a mesh node.
#[derive(Clone, Debug)]
pub struct TaskDescriptor {
    pub task_id:     u64,
    pub kernel_id:   String,
    pub priority:    TaskPriority,
    pub status:      TaskStatus,
    /// Target node — None means "any available".
    pub target_node: Option<NodeId>,
    /// Data payload size in bytes.
    pub payload_bytes: usize,
}

impl TaskDescriptor {
    pub fn new(task_id: u64, kernel_id: &str) -> Self {
        Self {
            task_id,
            kernel_id: alloc::string::ToString::to_string(kernel_id),
            priority:  TaskPriority::Normal,
            status:    TaskStatus::Pending,
            target_node: None,
            payload_bytes: 0,
        }
    }

    pub fn with_priority(mut self, p: TaskPriority) -> Self {
        self.priority = p; self
    }

    pub fn with_target(mut self, node: NodeId) -> Self {
        self.target_node = Some(node); self
    }

    pub fn with_payload(mut self, bytes: usize) -> Self {
        self.payload_bytes = bytes; self
    }
}

// ------------------------------------------------------------------
// MeshDispatcher — node registry + task queue
// ------------------------------------------------------------------

/// Manages a registry of mesh nodes and a task dispatch queue.
pub struct MeshDispatcher {
    nodes:      Vec<MeshNode>,
    task_queue: Vec<TaskDescriptor>,
    next_task:  u64,
}

impl MeshDispatcher {
    pub fn new() -> Self {
        Self {
            nodes:      Vec::new(),
            task_queue: Vec::new(),
            next_task:  1,
        }
    }

    /// Register a new node. Returns its NodeId.
    pub fn register_node(&mut self, node: MeshNode) -> NodeId {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Number of registered nodes.
    pub fn node_count(&self) -> usize { self.nodes.len() }

    /// Number of available nodes.
    pub fn available_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.available).count()
    }

    /// Submit a task. Returns its assigned task_id.
    pub fn submit(&mut self, kernel_id: &str) -> u64 {
        let id = self.next_task;
        self.next_task += 1;
        self.task_queue.push(TaskDescriptor::new(id, kernel_id));
        id
    }

    /// Submit a task with priority and payload.
    pub fn submit_full(
        &mut self,
        kernel_id: &str,
        priority: TaskPriority,
        payload_bytes: usize,
        target: Option<NodeId>,
    ) -> u64 {
        let id = self.next_task;
        self.next_task += 1;
        let mut task = TaskDescriptor::new(id, kernel_id)
            .with_priority(priority)
            .with_payload(payload_bytes);
        if let Some(node) = target {
            task = task.with_target(node);
        }
        self.task_queue.push(task);
        id
    }

    /// Number of pending tasks.
    pub fn pending_count(&self) -> usize {
        self.task_queue.iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count()
    }

    /// Mark a task as running.
    pub fn mark_running(&mut self, task_id: u64) -> bool {
        for t in &mut self.task_queue {
            if t.task_id == task_id {
                t.status = TaskStatus::Running;
                return true;
            }
        }
        false
    }

    /// Mark a task as complete.
    pub fn mark_complete(&mut self, task_id: u64) -> bool {
        for t in &mut self.task_queue {
            if t.task_id == task_id {
                t.status = TaskStatus::Complete;
                return true;
            }
        }
        false
    }

    /// Select the best available node for a task (highest capability, available).
    /// Priority: GPU > seL4 > CPU.
    pub fn select_node(&self, prefer_gpu: bool) -> Option<NodeId> {
        let available: Vec<&MeshNode> = self.nodes.iter()
            .filter(|n| n.available)
            .collect();
        if available.is_empty() { return None; }
        if prefer_gpu {
            if let Some(n) = available.iter().find(|n| n.capability == NodeCapability::Gpu) {
                return Some(n.id);
            }
        }
        if let Some(n) = available.iter().find(|n| n.capability == NodeCapability::Sel4) {
            return Some(n.id);
        }
        available.first().map(|n| n.id)
    }

    /// Drain all complete tasks from the queue.
    pub fn drain_complete(&mut self) -> Vec<TaskDescriptor> {
        let mut done = Vec::new();
        self.task_queue.retain(|t| {
            if t.status == TaskStatus::Complete {
                done.push(t.clone());
                false
            } else {
                true
            }
        });
        done
    }
}

impl Default for MeshDispatcher {
    fn default() -> Self { Self::new() }
}
