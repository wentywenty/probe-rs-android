use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use probe_rs::{
    Permissions,
    config::TargetSelector,
    flashing::{BinLoader, BinOptions, ElfLoader, ElfOptions, HexLoader, download_file},
    probe::{DebugProbeSelector, WireProtocol, list::Lister},
};

#[derive(Parser)]
#[command(name = "probe-rs-android")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List,
    Flash {
        #[arg(long)]
        chip: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "elf")]
        format: String,
        #[arg(long, default_value = "swd")]
        protocol: String,
        #[arg(long)]
        speed: Option<u32>,
        #[arg(long)]
        probe: Option<String>,
    },
}

fn open_probe(selector: Option<&str>) -> Result<probe_rs::probe::Probe> {
    let lister = Lister::new();
    let probes = lister.list_all();
    if probes.is_empty() {
        return Err(anyhow!("no probes found"));
    }
    let info = match selector {
        Some(s) => {
            let sel = DebugProbeSelector::from_str(s).with_context(|| format!("invalid probe selector: {s}"))?;
            probes
                .into_iter()
                .find(|p| sel.matches_probe(p))
                .ok_or_else(|| anyhow!("probe not found: {s}"))?
        }
        None => {
            if probes.len() != 1 {
                return Err(anyhow!("expected exactly one probe; found {}", probes.len()));
            }
            probes.into_iter().next().unwrap()
        }
    };
    info.open().context("failed to open probe")
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::List => {
            let probes = Lister::new().list_all_with_access();
            if probes.is_empty() {
                println!("no probes found");
            }
            for probe in probes {
                println!("{:?}", probe);
            }
        }
        Command::Flash {
            chip,
            file,
            format,
            protocol,
            speed,
            probe,
        } => {
            let mut probe = open_probe(probe.as_deref())?;
            let protocol = WireProtocol::from_str(&protocol)
                .map_err(|e| anyhow!("unknown protocol: {protocol}: {e}"))?;
            probe.select_protocol(protocol)?;
            if let Some(speed) = speed {
                probe.set_speed(speed)?;
            }
            let mut session = probe
                .attach(TargetSelector::from(chip.as_str()), Permissions::default())
                .with_context(|| format!("failed to attach to {chip}"))?;
            match format.as_str() {
                "elf" => download_file(&mut session, &file, ElfLoader(ElfOptions::default()))?,
                "hex" => download_file(&mut session, &file, HexLoader)?,
                "bin" => download_file(&mut session, &file, BinLoader(BinOptions::default()))?,
                other => return Err(anyhow!("unknown format: {other}")),
            }
            println!("flashed {} to {}", file.display(), chip);
        }
    }
    Ok(())
}
