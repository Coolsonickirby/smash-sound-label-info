# smash-sound-label-info

A Rust library for working with `soundlabelinfo.sli` files from Smash Ultimate.

### Example Usage

```rust
use sound_label_info::SliFile;

let mut file = SliFile::open("soundlabelinfo.sli")?;

for entry in file.entries() {
    println!("tone_name: {:#X}", entry.tone_name);
}

for entry in file.entries_mut() {
    entry.tone_id = 0;
}

file.save("soundlabelinfo.sli")?;
```
