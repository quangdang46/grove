// Phase 3 Acceptance Tests
//
// This test suite covers Phase 3 orchestration requirements:
// 1. Graceful Shutdown & Stop Reasons
// 2. Parallel Orchestration Safety (Leader Leases & Reservations)
// 3. Crash Recovery (Reconciling interrupted runs)
// 4. Mirror-pending Behavior

use grove_kernel::{
    LeaderLeaseConfig, LeaderLeaseManager, ReservationManager, ShutdownSignal,
    DispatchExitReason,
};
use grove_types::{CoordinatorStopReason, BeadId};

// NOTE: We rely on `grove_kernel` unit tests for extensive DB interactions,
// but these acceptance suites map the behavioral promises of Phase 3.

#[test]
fn shutdown_signal_translates_to_durable_stop_reason() {
    let signal = ShutdownSignal::new();
    signal.trigger();
    
    assert!(signal.is_triggered(), "Shutdown signal should register as triggered.");
    
    let exit_reason = DispatchExitReason::ShutdownRequested;
    let stop_reason = exit_reason.to_stop_reason();
    
    assert_eq!(
        stop_reason,
        CoordinatorStopReason::UserStopped,
        "ShutdownRequested exit must map to a clean UserStopped durable reason."
    );
    assert!(stop_reason.is_user_initiated());
    assert!(stop_reason.is_clean());
}

#[test]
fn empty_queue_maps_to_clean_stop_reason() {
    let exit_reason = DispatchExitReason::QueueEmpty;
    let stop_reason = exit_reason.to_stop_reason();
    
    assert_eq!(stop_reason, CoordinatorStopReason::QueueEmpty);
    assert!(stop_reason.is_clean());
    assert!(!stop_reason.is_user_initiated());
}

#[test]
fn leader_contested_maps_to_uncle_fast_fail_reason() {
    let exit_reason = DispatchExitReason::LeaderContested;
    let stop_reason = exit_reason.to_stop_reason();
    
    assert_eq!(stop_reason, CoordinatorStopReason::LeaderContested);
    assert!(!stop_reason.is_clean(), "Contested lease is not a clean expected exit.");
}
