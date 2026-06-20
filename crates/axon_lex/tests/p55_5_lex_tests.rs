// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P55.5 — axon_lex token completeness tests
// Verifies every v0.3 + sovereign/security/finance token is declared in token.ax

use axon_lex::token::Token;

// T1: v0.3 keyword tokens exist
#[test] fn test_actor_token()    { let _t = Token::Actor; }
#[test] fn test_handle_token()   { let _t = Token::Handle; }
#[test] fn test_intent_token()   { let _t = Token::Intent; }
#[test] fn test_opaque_token()   { let _t = Token::Opaque; }
#[test] fn test_foreach_token()  { let _t = Token::Foreach; }
#[test] fn test_yield_token()    { let _t = Token::Yield; }
#[test] fn test_uses_token()     { let _t = Token::Uses; }

// T2: provenance type tokens
#[test] fn test_tainted_token()  { let _t = Token::Tainted; }
#[test] fn test_clean_token()    { let _t = Token::Clean; }

// T3: security type tokens
#[test] fn test_secret_token()   { let _t = Token::Secret; }

// T4: sovereignty type tokens
#[test] fn test_auditable_token()  { let _t = Token::Auditable; }
#[test] fn test_expires_token()    { let _t = Token::Expires; }
#[test] fn test_resident_token()   { let _t = Token::Resident; }

// T5: finance type tokens
#[test] fn test_money_token()    { let _t = Token::Money; }
#[test] fn test_safeint_token()  { let _t = Token::SafeInt; }

// T6: v0.3 operator tokens
#[test] fn test_pipe_forward()   { let _t = Token::PipeForward; }
#[test] fn test_tilde_arrow()    { let _t = Token::TildeArrow; }
#[test] fn test_cap_bang()       { let _t = Token::CapBang; }
#[test] fn test_cap_question()   { let _t = Token::CapQuestion; }

// T7: decorator tokens
#[test] fn test_deterministic()      { let _t = Token::DecoratorDeterministic; }
#[test] fn test_constant_time()      { let _t = Token::DecoratorConstantTime; }
#[test] fn test_requires_consent()   { let _t = Token::DecoratorRequiresConsent; }
#[test] fn test_sealed_memory()      { let _t = Token::DecoratorSealedMemory; }
#[test] fn test_balanced()           { let _t = Token::DecoratorBalanced; }
#[test] fn test_atomic_financial()   { let _t = Token::DecoratorAtomicFinancial; }
#[test] fn test_model_signed()       { let _t = Token::DecoratorModelSigned; }
#[test] fn test_inference_budget()   { let _t = Token::DecoratorInferenceBudget; }
#[test] fn test_requires_human()     { let _t = Token::DecoratorRequiresHuman; }

// T8: temporal tokens
#[test] fn test_at_now()       { let _t = Token::AtNow; }
#[test] fn test_at_lifetime()  { let _t = Token::AtLifetime; }
#[test] fn test_at_epoch()     { let _t = Token::AtEpoch; }

// T9: token classification functions
#[test]
fn test_sovereign_type_classification() {
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Tainted));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Clean));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Secret));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Auditable));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Expires));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Resident));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::Money));
    assert!(axon_lex::token::token_is_sovereign_type(&Token::SafeInt));
    assert!(!axon_lex::token::token_is_sovereign_type(&Token::Fn));
}

// T10: decorator classification
#[test]
fn test_decorator_classification() {
    assert!(axon_lex::token::token_is_decorator(&Token::DecoratorDeterministic));
    assert!(axon_lex::token::token_is_decorator(&Token::DecoratorConstantTime));
    assert!(axon_lex::token::token_is_decorator(&Token::DecoratorBalanced));
    assert!(axon_lex::token::token_is_decorator(&Token::DecoratorAtomicFinancial));
    assert!(!axon_lex::token::token_is_decorator(&Token::Fn));
}
