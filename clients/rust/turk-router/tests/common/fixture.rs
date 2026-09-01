//! The fixture text format, parsed and rendered byte for byte.
//!
//! A fixture is one pool's window as the program's own test harness captured it: a header, then
//! one block per window slot after the venue program, then the mint accounts as extras. Slot
//! roles say what the harness substitutes at run time (`in_ata`, `out_ata`, `payer`) and what it
//! installs as captured (`fixed`, which carries a body) or references by address (`program_ref`).

use solana_pubkey::Pubkey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixture {
    pub kind: String,
    pub pool_b58: String,
    pub slot: u64,
    pub unix_ts: i64,
    pub hop_kind: u8,
    pub program_id: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_token_program: Pubkey,
    pub output_token_program: Pubkey,
    pub hook_lens: [u8; 2],
    pub slots: Vec<Slot>,
    pub extras: Vec<Extra>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Fixed,
    InAta,
    OutAta,
    Payer,
    ProgramRef,
}

impl Role {
    fn parse(text: &str) -> Role {
        match text {
            "fixed" => Role::Fixed,
            "in_ata" => Role::InAta,
            "out_ata" => Role::OutAta,
            "payer" => Role::Payer,
            "program_ref" => Role::ProgramRef,
            other => panic!("unknown role {other}"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Role::Fixed => "fixed",
            Role::InAta => "in_ata",
            Role::OutAta => "out_ata",
            Role::Payer => "payer",
            Role::ProgramRef => "program_ref",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub owner: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub writable: bool,
    pub signer: bool,
    pub role: Role,
    pub pubkey: Pubkey,
    pub body: Option<Body>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extra {
    pub role: String,
    pub pubkey: Pubkey,
    pub body: Body,
}

impl Fixture {
    /// The window the program sees: the venue program, then every slot.
    pub fn window_pubkeys(&self) -> Vec<Pubkey> {
        std::iter::once(self.program_id)
            .chain(self.slots.iter().map(|slot| slot.pubkey))
            .collect()
    }

    pub fn slot(&self, index: usize) -> &Slot {
        self.slots
            .get(index)
            .unwrap_or_else(|| panic!("fixture has no slot {index}"))
    }

    pub fn extra(&self, role: &str) -> &Extra {
        self.extras
            .iter()
            .find(|extra| extra.role == role)
            .unwrap_or_else(|| panic!("fixture has no extra {role}"))
    }
}

enum Target {
    Header,
    Slot,
    Extra,
}

pub fn parse(text: &str) -> Fixture {
    let mut kind = None;
    let mut pool_b58 = None;
    let mut slot = None;
    let mut unix_ts = None;
    let mut hop_kind = None;
    let mut hook_lens = [0u8, 0u8];
    let mut program_id = None;
    let mut input_mint = None;
    let mut output_mint = None;
    let mut input_token_program = None;
    let mut output_token_program = None;
    let mut slots: Vec<Slot> = Vec::new();
    let mut extras: Vec<Extra> = Vec::new();

    let mut target = Target::Header;
    let mut pubkey: Option<Pubkey> = None;
    let mut owner: Option<Pubkey> = None;
    let mut lamports: Option<u64> = None;
    let mut data: Option<Vec<u8>> = None;

    fn flush(
        target: &Target,
        slots: &mut [Slot],
        extras: &mut [Extra],
        pubkey: &mut Option<Pubkey>,
        owner: &mut Option<Pubkey>,
        lamports: &mut Option<u64>,
        data: &mut Option<Vec<u8>>,
    ) {
        match target {
            Target::Header => {}
            Target::Slot => {
                let entry = slots.last_mut().expect("account header seen");
                entry.pubkey = pubkey.take().expect("account block has pubkey");
                entry.body = match (owner.take(), lamports.take(), data.take()) {
                    (Some(owner), Some(lamports), Some(data)) => Some(Body {
                        owner,
                        lamports,
                        data,
                    }),
                    (None, None, None) => None,
                    _ => panic!("partial account body"),
                };
            }
            Target::Extra => {
                let entry = extras.last_mut().expect("extra header seen");
                entry.pubkey = pubkey.take().expect("extra block has pubkey");
                entry.body = Body {
                    owner: owner.take().expect("extra has owner"),
                    lamports: lamports.take().expect("extra has lamports"),
                    data: data.take().expect("extra has data"),
                };
            }
        }
    }

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let (key, value) = match line.split_once(':') {
            Some((key, value)) => (key.trim(), value.trim()),
            None => (line, ""),
        };
        match key {
            "account" => {
                flush(
                    &target,
                    &mut slots,
                    &mut extras,
                    &mut pubkey,
                    &mut owner,
                    &mut lamports,
                    &mut data,
                );
                target = Target::Slot;
                slots.push(Slot {
                    writable: header_field(value, "writable=") == "true",
                    signer: header_field(value, "signer=") == "true",
                    role: Role::parse(header_field(value, "role=")),
                    pubkey: Pubkey::default(),
                    body: None,
                });
            }
            "extra" => {
                flush(
                    &target,
                    &mut slots,
                    &mut extras,
                    &mut pubkey,
                    &mut owner,
                    &mut lamports,
                    &mut data,
                );
                target = Target::Extra;
                extras.push(Extra {
                    role: header_field(value, "role=").to_string(),
                    pubkey: Pubkey::default(),
                    body: Body {
                        owner: Pubkey::default(),
                        lamports: 0,
                        data: Vec::new(),
                    },
                });
            }
            "pubkey" => pubkey = Some(parse_key(value)),
            "owner" => owner = Some(parse_key(value)),
            "lamports" => lamports = Some(value.parse().expect("lamports u64")),
            "data_hex" => data = Some(hex_decode(value)),
            "hop_kind" => hop_kind = Some(value.parse().expect("hop_kind u8")),
            "hook_lens" => {
                let (a, b) = value.split_once(',').expect("hook_lens is 'A,B'");
                hook_lens = [
                    a.trim().parse().expect("hook_lens[0] u8"),
                    b.trim().parse().expect("hook_lens[1] u8"),
                ];
            }
            "slot" => slot = Some(value.parse().expect("slot u64")),
            "unix_ts" => unix_ts = Some(value.parse().expect("unix_ts i64")),
            "program_id" => program_id = Some(parse_key(value)),
            "input_mint" => input_mint = Some(parse_key(value)),
            "output_mint" => output_mint = Some(parse_key(value)),
            "input_token_program" => input_token_program = Some(parse_key(value)),
            "output_token_program" => output_token_program = Some(parse_key(value)),
            "kind" => kind = Some(value.to_string()),
            "pool_b58" => pool_b58 = Some(value.to_string()),
            _ => {}
        }
    }
    flush(
        &target,
        &mut slots,
        &mut extras,
        &mut pubkey,
        &mut owner,
        &mut lamports,
        &mut data,
    );

    Fixture {
        kind: kind.expect("kind header"),
        pool_b58: pool_b58.expect("pool_b58 header"),
        slot: slot.expect("slot header"),
        unix_ts: unix_ts.expect("unix_ts header"),
        hop_kind: hop_kind.expect("hop_kind header"),
        program_id: program_id.expect("program_id header"),
        input_mint: input_mint.expect("input_mint header"),
        output_mint: output_mint.expect("output_mint header"),
        input_token_program: input_token_program.expect("input_token_program header"),
        output_token_program: output_token_program.expect("output_token_program header"),
        hook_lens,
        slots,
        extras,
    }
}

pub fn render(fixture: &Fixture) -> String {
    let mut out = String::new();
    out.push_str(&format!("kind: {}\n", fixture.kind));
    out.push_str(&format!("pool_b58: {}\n", fixture.pool_b58));
    out.push_str(&format!("slot: {}\n", fixture.slot));
    out.push_str(&format!("unix_ts: {}\n", fixture.unix_ts));
    out.push_str(&format!("hop_kind: {}\n", fixture.hop_kind));
    out.push_str(&format!(
        "program_id: {}\n",
        format_key(&fixture.program_id)
    ));
    out.push_str(&format!(
        "input_mint: {}\n",
        format_key(&fixture.input_mint)
    ));
    out.push_str(&format!(
        "output_mint: {}\n",
        format_key(&fixture.output_mint)
    ));
    out.push_str(&format!(
        "input_token_program: {}\n",
        format_key(&fixture.input_token_program)
    ));
    out.push_str(&format!(
        "output_token_program: {}\n",
        format_key(&fixture.output_token_program)
    ));
    out.push_str(&format!("accounts: {}\n", fixture.slots.len()));
    if fixture.hook_lens != [0, 0] {
        out.push_str(&format!(
            "hook_lens: {},{}\n",
            fixture.hook_lens[0], fixture.hook_lens[1]
        ));
    }
    for (index, slot) in fixture.slots.iter().enumerate() {
        out.push_str(&format!(
            "account: index={index} writable={} signer={} role={}\n",
            slot.writable,
            slot.signer,
            slot.role.name()
        ));
        out.push_str(&format!("pubkey: {}\n", format_key(&slot.pubkey)));
        if let Some(body) = &slot.body {
            render_body(&mut out, body);
        }
    }
    for extra in &fixture.extras {
        out.push_str(&format!("extra: role={}\n", extra.role));
        out.push_str(&format!("pubkey: {}\n", format_key(&extra.pubkey)));
        render_body(&mut out, &extra.body);
    }
    out
}

fn render_body(out: &mut String, body: &Body) {
    out.push_str(&format!("owner: {}\n", format_key(&body.owner)));
    out.push_str(&format!("lamports: {}\n", body.lamports));
    out.push_str(&format!("data_hex: {}\n", hex_encode(&body.data)));
}

fn header_field<'a>(header: &'a str, key: &str) -> &'a str {
    let from = header.find(key).expect("header field") + key.len();
    header[from..]
        .split_whitespace()
        .next()
        .expect("header field value")
}

fn parse_key(value: &str) -> Pubkey {
    let inner = value
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .expect("a bracketed key");
    let bytes: Vec<u8> = inner
        .split(',')
        .map(|token| token.trim().parse().expect("a byte"))
        .collect();
    Pubkey::new_from_array(<[u8; 32]>::try_from(bytes).expect("32 bytes"))
}

fn format_key(key: &Pubkey) -> String {
    let joined = key
        .to_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

fn hex_decode(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "odd-length hex");
    (0..text.len() / 2)
        .map(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).expect("hex byte"))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
