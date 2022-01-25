use std::time::SystemTime;

pub struct Authcode {
    pub user_email: String,
    pub code: String,
    pub timer: SystemTime,
}

impl Authcode {
    pub fn new(user_email: String, code: String) -> Self {
        Self {
            user_email,
            code,
            timer: SystemTime::now(),
        }
    }

    pub fn expired(&self) -> bool {
        let r = matches!(self.timer.elapsed(), Ok(elapsed) if elapsed.as_secs() >  60 * 5);

        log::debug!(
            "Authcode::expired {} => {}s",
            self.code,
            self.timer.elapsed().unwrap_or_default().as_secs()
        );

        r
    }
}
