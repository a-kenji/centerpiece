use anyhow::Context;

use crate::plugin::utils::Plugin;

pub struct SystemPlugin {
    entries: Vec<crate::model::Entry>,
}

impl Plugin for SystemPlugin {
    fn new() -> Self {
        return Self { entries: vec![] };
    }

    fn id() -> &'static str {
        return "system";
    }

    fn priority() -> u32 {
        return 15;
    }

    fn title() -> &'static str {
        return "󰌢 System";
    }

    fn update_entries(&mut self) -> anyhow::Result<()> {
        self.entries.clear();

        self.entries = vec![
            crate::model::Entry {
                id: String::from("lock"),
                title: String::from("Lock"),
                action: String::from(""),
                meta: String::from("System"),
                command: Some(vec![String::from("lock")]),
            },
            crate::model::Entry {
                id: String::from("restart"),
                title: String::from("Restart"),
                action: String::from(""),
                meta: String::from("System"),
                command: Some(vec![String::from("reboot")]),
            },
            crate::model::Entry {
                id: String::from("shutdown"),
                title: String::from("Shutdown"),
                action: String::from(""),
                meta: String::from("System"),
                command: Some(vec![String::from("poweroff")]),
            },
            crate::model::Entry {
                id: String::from("sleep"),
                title: String::from("Sleep"),
                action: String::from(""),
                meta: String::from("System Hibernate Suspend"),
                command: Some(vec![String::from("systemctl suspend")]),
            },
        ];

        return Ok(());
    }

    fn entries(&self) -> Vec<crate::model::Entry> {
        return self.entries.clone();
    }

    fn activate(
        &mut self,
        entry: crate::model::Entry,
        plugin_channel_out: &mut iced::futures::channel::mpsc::Sender<crate::Message>,
    ) -> anyhow::Result<()> {
        let command = entry.command.context(format!(
            "Failed to unpack command while activating entry with id '{}'.",
            entry.id
        ))?;
        std::process::Command::new(&command[0])
            .args(&command[1..])
            .spawn()?;

        plugin_channel_out
            .try_send(crate::Message::Exit)
            .context(format!(
                "Failed to send message to exit application while activating entry with id '{}'.",
                entry.id
            ))?;

        return Ok(());
    }
}
