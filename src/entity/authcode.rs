pub const MAX_AGE: u64 = 60 * 2;

pub struct Authcode {
    pub user_email: String,
    pub code: String,
}

impl Authcode {
    pub fn new(user_email: String, code: String) -> Self {
        Self { user_email, code }
    }

    /* pub fn expired(&self) -> bool {
        let r = matches!(self.timer.elapsed(), Ok(elapsed) if elapsed.as_secs() >  MAX_AGE);

        log::debug!(
            "Authcode::expired {} => {}s",
            self.code,
            self.timer.elapsed().unwrap_or_default().as_secs()
        );

        r
    } */
}
