fn main() {
    let src = std::path::Path::new("fixtures/library/Helloween/Giants & Monsters (2021)/01 - Silent Echoes.flac");
    let dst = std::path::Path::new("/tmp/opencode/bare.flac");
    std::fs::copy(src, dst).unwrap();
    let mut tagged = lofty::read_from_path(dst).unwrap();
    println!("before: {:?}", tagged.tags().map(|t| t.tag_type()).collect::<Vec<_>>());
    for tt in [lofty::tag::TagType::VorbisComments, lofty::tag::TagType::Id3v2, lofty::tag::TagType::Id3v1, lofty::tag::TagType::Ape] {
        let r = tagged.remove(tt);
        println!("remove {tt:?} -> {:?}", r.as_ref().map(|t| t.is_empty()));
    }
    tagged.save_to_path(dst, lofty::config::WriteOptions::default()).unwrap();
    let re = lofty::read_from_path(dst).unwrap();
    println!("after: {:?}", re.tags().map(|t| (t.tag_type(), t.len())).collect::<Vec<_>>());
}
