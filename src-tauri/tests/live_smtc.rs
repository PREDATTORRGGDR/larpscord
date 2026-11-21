//! Живой тест SMTC: что сейчас играет на этой машине.
//! cargo test --test live_smtc -- --ignored --nocapture

#[tokio::test]
#[ignore]
async fn live_smtc_snapshot() {
    let snap = larpscord_lib::music::smtc::snapshot_async().await;
    println!("available: {}", snap.available);
    println!("source:    {:?}", snap.source);
    println!("title:     {:?}", snap.title);
    println!("artist:    {:?}", snap.artist);
    println!("album:     {:?}", snap.album);
    println!("playing:   {}", snap.playing);
    println!("position:  {:?}", snap.position_secs);
    println!("duration:  {:?}", snap.duration_secs);
    println!("thumbnail: {} байт", snap.thumbnail.as_ref().map(|t| t.len()).unwrap_or(0));
}
