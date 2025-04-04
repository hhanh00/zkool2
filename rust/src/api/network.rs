use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn set_lwd(lwd: &str) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_lwd(lwd);
}
