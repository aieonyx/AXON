// axon_ai::sfl — Semantic Frame Locks. SPEC: 6A-05
// Reading frame integrity: frame shifts across async/IPC boundaries
// without @frame_transition are hard compile errors.

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum CapContext{
    UserSpace,
    SeL4IPC,
    KernelMode,
    AIInference,
    Custom(String),
}

impl std::fmt::Display for CapContext{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        match self{
            CapContext::UserSpace=>write!(f,"user-space"),
            CapContext::SeL4IPC=>write!(f,"seL4-ipc"),
            CapContext::KernelMode=>write!(f,"kernel-mode"),
            CapContext::AIInference=>write!(f,"ai-inference"),
            CapContext::Custom(s)=>write!(f,"{}",s),
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TrustLevel{Untrusted,User,Sovereign,Kernel}

#[derive(Debug,Clone)]
pub struct SemanticFrame{
    pub capability_context:CapContext,
    pub trust_level:TrustLevel,
    pub async_depth:u32,
    pub ipc_boundary:bool,
}

impl SemanticFrame{
    pub fn user_space()->Self{
        Self{capability_context:CapContext::UserSpace,trust_level:TrustLevel::User,async_depth:0,ipc_boundary:false}
    }
    pub fn crosses_boundary(&self,other:&Self)->bool{
        self.capability_context!=other.capability_context||
        self.trust_level!=other.trust_level||
        self.ipc_boundary!=other.ipc_boundary
    }
}

#[derive(Debug,Clone)]
pub struct FrameShiftViolation{
    pub fn_name:String,
    pub from_context:String,
    pub to_context:String,
    pub message:String,
}

impl std::fmt::Display for FrameShiftViolation{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"FrameShiftViolation in fn {}: frame crossed [{} -> {}] without @frame_transition",
            self.fn_name,self.from_context,self.to_context)
    }
}

/// Phase 6 SFL analyser — source-level frame shift detection.
/// PHASE7: full AST-level SemanticFrame propagation through all call sites.
/// SPEC: 6A-05
pub struct SFLAnalyser;

const SEL4_PATTERNS:&[&str]=&["seL4::","sel4::","ipc::send","ipc::recv","ipc::call","seL4_Call","seL4_Send"];

impl SFLAnalyser{
    /// Analyse AXON source for frame shift violations.
    /// Detects async functions crossing seL4 IPC boundaries without @frame_transition.
    /// SPEC: 6A-05
    pub fn analyse(source:&str)->Vec<FrameShiftViolation>{
        let mut violations=Vec::new();
        let lines:Vec<&str>=source.lines().collect();
        let mut i=0;
        while i<lines.len(){
            let line=lines[i].trim();
            // Detect async fn without @frame_transition
            if line.starts_with("async fn ")||line.starts_with("task "){
                let fn_name=Self::extract_fn_name(line);
                let has_frame_transition=i>0&&lines[..i].iter().rev()
                    .take(5).any(|l|l.trim().starts_with("@frame_transition"));
                // Scan function body for seL4 IPC patterns
                let body_start=i+1;
                let body_end=(body_start+50).min(lines.len());
                let body_lines=&lines[body_start..body_end];
                let has_ipc=body_lines.iter().any(|l|{
                    SEL4_PATTERNS.iter().any(|p|l.contains(p))
                });
                let has_await=body_lines.iter().any(|l|l.contains(".await"));
                if has_ipc&&has_await&&!has_frame_transition{
                    violations.push(FrameShiftViolation{
                        fn_name:fn_name.clone(),
                        from_context:"user-space".to_string(),
                        to_context:"seL4-ipc".to_string(),
                        message:format!(
                            "async fn {} crosses seL4-ipc boundary without @frame_transition",
                            fn_name),
                    });
                }
            }
            i+=1;
        }
        violations
    }

    /// Verify that a @frame_transition decorator has valid from/to contexts.
    /// SPEC: 6A-05
    pub fn validate_transition(from:&str,to:&str)->bool{
        let valid_contexts=["user-space","seL4-ipc","kernel-mode","ai-inference"];
        valid_contexts.contains(&from)&&valid_contexts.contains(&to)&&from!=to
    }

    fn extract_fn_name(line:&str)->String{
        let rest=line.trim_start_matches("async fn ").trim_start_matches("task ");
        rest.split('(').next().unwrap_or("").trim().to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_no_violation_with_frame_transition(){
        let src="@frame_transition(from: user-space, to: seL4-ipc)
async fn request_cap():
    seL4::ipc::send(req).await
";
        let v=SFLAnalyser::analyse(src);
        assert!(v.is_empty(),"should be no violation with @frame_transition");
    }

    #[test]
    fn test_violation_without_frame_transition(){
        let src="async fn request_cap():
    seL4::ipc::send(req).await
";
        let v=SFLAnalyser::analyse(src);
        assert!(!v.is_empty(),"should detect frame shift violation");
        assert_eq!(v[0].fn_name,"request_cap");
    }

    #[test]
    fn test_no_violation_same_frame(){
        let src="async fn process_data():
    transform(data).await
";
        let v=SFLAnalyser::analyse(src);
        assert!(v.is_empty(),"same-frame async should not violate");
    }

    #[test]
    fn test_violation_message_contains_fn_name(){
        let src="async fn send_ipc_call():
    seL4::ipc::call(msg).await
";
        let v=SFLAnalyser::analyse(src);
        assert!(!v.is_empty());
        assert!(v[0].message.contains("send_ipc_call"));
    }

    #[test]
    fn test_violation_display(){
        let v=FrameShiftViolation{
            fn_name:"f".to_string(),
            from_context:"user-space".to_string(),
            to_context:"seL4-ipc".to_string(),
            message:"test".to_string(),
        };
        assert!(v.to_string().contains("FrameShiftViolation"));
        assert!(v.to_string().contains("user-space"));
    }

    #[test]
    fn test_semantic_frame_crosses_boundary(){
        let a=SemanticFrame::user_space();
        let b=SemanticFrame{capability_context:CapContext::SeL4IPC,trust_level:TrustLevel::Kernel,async_depth:1,ipc_boundary:true};
        assert!(a.crosses_boundary(&b));
    }

    #[test]
    fn test_same_frame_no_boundary(){
        let a=SemanticFrame::user_space();
        let b=SemanticFrame::user_space();
        assert!(!a.crosses_boundary(&b));
    }

    #[test]
    fn test_validate_transition_valid(){
        assert!(SFLAnalyser::validate_transition("user-space","seL4-ipc"));
        assert!(SFLAnalyser::validate_transition("user-space","ai-inference"));
    }

    #[test]
    fn test_validate_transition_same_context_invalid(){
        assert!(!SFLAnalyser::validate_transition("user-space","user-space"));
    }

    #[test]
    fn test_task_ipc_violation(){
        let src="task monitor_ipc():
    sel4::ipc::recv(msg).await
";
        let v=SFLAnalyser::analyse(src);
        assert!(!v.is_empty(),"task with IPC without frame_transition should violate");
    }
}
