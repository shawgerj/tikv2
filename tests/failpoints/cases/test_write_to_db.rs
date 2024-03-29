//use std::sync::atomic::AtomicBool;
//use std::sync::{mpsc, Arc, Mutex};
//use std::time::Duration;

use test_raftstore::*;
use tikv_util::config::*;

fn run_write_to_db_with_failpoint(fp: &str) {
    let mut cluster = new_fault_node_cluster(0, 1);
    cluster.cfg.raft_store.raft_base_tick_interval = ReadableDuration::millis(500);
    cluster.cfg.raft_store.raft_max_size_per_msg = ReadableSize(5);
    cluster.cfg.raft_store.raft_election_timeout_ticks = 50;
    cluster.cfg.raft_store.max_leader_missing_duration = ReadableDuration::hours(1);
    cluster.cfg.raft_store.peer_stale_state_check_interval = ReadableDuration::minutes(30);
    cluster.cfg.raft_store.abnormal_leader_missing_duration = ReadableDuration::hours(1);
    // disable compact log to make test more stable.
    cluster.cfg.raft_store.raft_log_gc_threshold = 3000;
    cluster.pd_client.disable_default_operator();
    cluster.run_conf_change();

    fail::cfg(fp, "return").unwrap();

    for i in 0..10 {
        let v = format!("{:04}", i);
        cluster.must_put(v.as_bytes(), v.as_bytes());
    }
    must_get_none(&cluster.get_engine(1), b"0001");
    must_get_none(&cluster.get_engine(1), b"0009");

    cluster.stop_node(1);
    cluster.reboot_engine(1);
    must_get_none(&cluster.get_engine(1), b"0001");
    must_get_none(&cluster.get_engine(1), b"0009");
    fail::remove(fp);
    cluster.run_node(1).unwrap(); // reapply raft log on startup
    sleep_ms(2000); // race to catch up raft log before reading from engine...

    must_get_equal(&cluster.get_engine(1), b"0001", b"0001");
    must_get_equal(&cluster.get_engine(1), b"0009", b"0009");
}

#[test]
fn test_fail_write_to_db_begin() {
    run_write_to_db_with_failpoint("write_to_db_begin");
}

#[test]
fn test_fail_write_to_db_after_ingest_sst() {
    run_write_to_db_with_failpoint("write_to_db_after_ingest_sst");
}

#[test]
fn test_fail_write_to_db_after_write_wb() {
    run_write_to_db_with_failpoint("write_to_db_after_write_wb");
}

#[test]
fn test_fail_write_to_db_after_sync_memtables() {
    run_write_to_db_with_failpoint("write_to_db_after_sync_memtables");
}

#[test]
fn test_fail_write_to_db_after_delete_ssts() {
    run_write_to_db_with_failpoint("write_to_db_after_delete_ssts");
}
