"""Read-only transition prerequisites; this module cannot start or stop any process."""
SOURCE = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
CANDIDATE = "be7395acfce0ec1e36b83300b576db408f8c35bb"
OWNER_TERMINAL = "observed-real-next-successful-deadlines-requires-final-audit"

def evaluate_transition(initial, cycle_audit, monitor, cleanup, session, processes, desktops, preferences):
    if monitor["sourceCommit"] != SOURCE or session["sourceCommit"] != SOURCE:
        raise ValueError("Original lifecycle source differs")
    if monitor["applicationPid"] != 37276 or monitor["monitorPid"] != 3920 or session["supervisorPid"] != 24300:
        raise ValueError("Original lifecycle owner identity differs")
    if cycle_audit["sourceCommit"] != SOURCE or session["applicationPid"] != 37276:
        raise ValueError("Observation/session identity differs")
    blocked = []
    if cycle_audit["status"] != "recorded_next_successes_require_terminal_and_scope_review":
        blocked.append("original stable/panel persisted next successes remain unproved")
    if monitor["status"] != OWNER_TERMINAL or not monitor.get("finishedAtUtc"):
        blocked.append("original monitor has no reviewed successful terminal decision and finished timestamp")
    if monitor.get("failedObservations", 0) or monitor.get("earlierThanRecordedDeadlineTransitionObserved"):
        blocked.append("original monitor observation gaps or early transition require separate review")
    if not monitor.get("originalUpdatePreferencesRestored") or monitor.get("cleanupError"):
        blocked.append("original monitor cleanup has not restored its initial update preferences")
    original = initial["result"]
    expected = {"channel": original["originalChannel"], "automaticCheck": original["originalAutomaticCheck"]}
    if cleanup is None:
        blocked.append("original native cleanup receipt is absent")
    else:
        if cleanup["sourceCommit"] != SOURCE or cleanup["applicationPid"] != 37276:
            raise ValueError("Cleanup belongs to another lifecycle")
        result = cleanup["result"]
        if not result.get("originalUpdatePreferencesRestored") or {
            "channel": result.get("channel"), "automaticCheck": result.get("automaticCheck")
        } != expected:
            blocked.append("native cleanup readback differs from the original update preference pair")
    if preferences != expected:
        blocked.append("actual isolated disk update preferences do not match the original pair")
    if session["status"] not in {"application-exited", "stop-requested"} or not session.get("finishedAtUtc"):
        blocked.append("original application supervisor has not completed its own cleanup")
    if {value["pid"] for value in processes} != {37276, 24300, 3920}:
        raise ValueError("Actual process readback is incomplete")
    for value in processes:
        if value["actualLiveHandle"]:
            blocked.append(f"original process {value['pid']} is still live")
        elif not value.get("exactCreationMatched") and value.get("queryError") != 87:
            # ERROR_INVALID_PARAMETER is the documented absent-PID result.
            # Access denial and other query failures cannot masquerade as stopped.
            blocked.append(f"original process {value['pid']} cannot be confirmed stopped")
    if desktops["count"] != 0:
        blocked.append("independent whole-host desktop-process count is not zero")
    return {
        "status": "pending_original_owners_or_zero_instance" if blocked else "read_only_prerequisites_met_requires_operator_review",
        "blockingConditions": blocked, "currentLifecycleSource": SOURCE, "candidateSource": CANDIDATE,
        "automaticProcessTransitionAllowed": False, "foregroundAuthorityGranted": False,
        "ownerToolTerminalResultsStillNeedReview": True,
        "localCandidateIdentityMustBeRecheckedImmediatelyBeforeStart": True,
        "candidateNativeSourceMatchedByThisOldCycle": False,
        "rawNetworkOrQueryCountAccepted": False, "earliestQueryStartElapsedProven": False,
        "automaticTriggerProven": False, "originalTaskRequiredChecksRecovered": False,
        "realFullCycleAccepted": False, "betaCycleAccepted": False, "pairedPerformanceAccepted": False,
        "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    }
