use bgpkit_broker::{BgpkitBroker, BrokerItem};
use bgpkit_parser::models::AsPathSegment;
use bgpkit_parser::{BgpElem, BgpkitParser};
use crossbeam_channel::{unbounded, Receiver};
use ipnet::IpNet;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use threadpool::ThreadPool;

/// returns route server asns with associated names
fn setup_target_asns() -> HashMap<u32, String> {
    let mut target_asns: HashMap<u32, String> = HashMap::new();
    target_asns.insert(6695, "DE-CIX Frankfurt".to_string());
    target_asns.insert(6777, "AMS-IX".to_string());
    target_asns.insert(51706, "France-IX Paris".to_string());
    target_asns.insert(34307, "NL-ix".to_string());
    target_asns.insert(37195, "NapAfrica".to_string());
    target_asns.insert(8714, "LINX".to_string());
    target_asns.insert(33108, "SIX".to_string());
    target_asns.insert(50952, "DATAIX".to_string());
    target_asns.insert(8631, "MSK-IX Moscow".to_string());
    target_asns.insert(61968, "MIX-IT".to_string());
    target_asns.insert(4635, "HKIX".to_string());
    target_asns.insert(32184, "Any2".to_string());
    target_asns.insert(33108, "SIX".to_string());
    target_asns.insert(13538, "NYIIX".to_string());
    target_asns.insert(11670, "TorIX".to_string());
    target_asns.insert(49869, "PiterIX".to_string());
    target_asns.insert(52005, "Netnod".to_string());
    target_asns.insert(31210, "DTEL-IX".to_string());
    target_asns.insert(63034, "DE-CIX New York".to_string());
    target_asns.insert(42476, "SwissIX".to_string());
    target_asns.insert(47200, "NIC.CZ".to_string());
    target_asns.insert(26162, "IX.br".to_string());
    target_asns
}

/// obataints list of available route collectors at timestamp, sorted descending by rib size.
fn bgpkit_get_ribs_size_ordered(ts: i64) -> Vec<BrokerItem> {
    let broker = BgpkitBroker::new()
        .ts_start(&ts.to_string())
        .ts_end(&ts.to_string())
        .data_type("rib");

    broker
        .into_iter()
        .sorted_by_key(|item| -item.rough_size)
        .collect()
}

/// extracts the AS path as Vec<u32> from a BgpElem
fn extract_simplified_path(elem: &BgpElem) -> Option<Vec<u32>> {
    // facilitate the complex path object into a dense u32 vector
    let mut path_dense: Vec<u32> = Vec::new();
    for segment in &elem.as_path.as_ref().unwrap().segments {
        match segment {
            AsPathSegment::AsSequence(sequence) => {
                for asn_obj in sequence {
                    // is this path prepending? If so, skip adding the asn again.
                    if !path_dense.is_empty() {
                        // unwrap only safe if nested. No lazy evaluation.
                        if asn_obj.asn.eq(path_dense.last().unwrap()) {
                            continue;
                        }
                    }
                    path_dense.push(asn_obj.asn as u32);
                }
            }
            _ => return None, // contains AS_SETs, discurraged & soon-depreciated.
        }
    }
    Some(path_dense)
}

/// Summarizes and displays the (member, route server, prefix)-triplets prodocued by the workers
fn run_consumer_and_print(ch_in: Receiver<(u32, u32, IpNet)>, target_rses: &HashMap<u32, String>) {
    let mut members_per_rs: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut pfx_per_rs: HashMap<u32, HashSet<IpNet>> = HashMap::new();

    // get all the triplets from the workers
    loop {
        if let Ok((member, rs, pfx)) = ch_in.recv() {
            members_per_rs.entry(rs).or_default().insert(member);
            pfx_per_rs.entry(rs).or_default().insert(pfx);
        } else {
            break;
        }
    }

    println!(
        "{0: <20} | {1: <8 } | {2: <20} | {3: <25} | {4: <25}",
        "IX", "RS ASN", "re-appending members", "IPv4 Prefixes involved", "IPv6 Prefixes involved"
    );
    println!("{0:-<20}-+-{1:-<8 }-+-{2:-<20}-+-{3:-<25}-+-{4:-<25}",
        "", "", "", "", "");
    // produce & display summary.
    for (asn, name) in target_rses.iter().sorted_by_key(|&(asn, _)| -(members_per_rs.entry(*asn).or_default().len() as i32)) {
        let cnt_mem: usize = members_per_rs.entry(*asn).or_default().len();
        let cnt_pfx_v4: usize = pfx_per_rs
            .entry(*asn)
            .or_default()
            .iter()
            .filter(|p| match p {
                IpNet::V4(_) => true,
                _ => false,
            })
            .count();
        let cnt_pfx_v6: usize = pfx_per_rs
            .entry(*asn)
            .or_default()
            .iter()
            .filter(|p| match p {
                IpNet::V6(_) => true,
                _ => false,
            })
            .count();
        println!(
            "{0: <20} | {1: <8 } | {2: <20} | {3: <25} | {4: <25}",
            name, asn, cnt_mem, cnt_pfx_v4, cnt_pfx_v6
        );
    }
}

fn main() {

    // configuration
    let num_workers = 20;
        let rib_ts = 1690214400; // 2023-07-24, 16:00:00 UTC
    let target_asns = setup_target_asns();

    // worker pool and channels for connecting
    let pool = ThreadPool::new(num_workers);
    let (ch_out, ch_in) = unbounded();

    // get size-ordered broker items at rib ts
    let broker_items = bgpkit_get_ribs_size_ordered(rib_ts);

    // add one worker thread per collector
    for target in broker_items {

        // ensure needed structures are cloned and ready to move into closure
        let ch_out_cl = ch_out.clone();
        let target_asns_cl = target_asns.clone();

        // enqueue the spawn of a new thread
        pool.execute(move || {
            // closure that processes the data of a single route collector.
            let parser = BgpkitParser::new(target.url.as_str()).unwrap();

            // iterate through elements
            for elem in parser.into_elem_iter() {
                // extract path (if not AS_SET or empty)
                if let Some(path) = extract_simplified_path(&elem) {
                    for i in 1..path.len() {
                        // find RS ASNs and push their data to consumer.
                        if target_asns_cl.contains_key(&path[i]) {
                            ch_out_cl
                                .send((path[i - 1], path[i], elem.prefix.prefix.clone()))
                                .unwrap();
                        }
                    }
                }
            }
        });
    }

    // we delivered clones to all threads, so we still have to drop the initial reference.
    drop(ch_out);

    // collects results & produces summary.
    run_consumer_and_print(ch_in, &target_asns);
}
