#![cfg(feature = "test_local")]

use std::path::Path;

use sqpack::{Package, Result, SqPackPackage};

#[tokio::test]
async fn read_test() -> Result<()> {
    let _ = pretty_env_logger::formatted_timed_builder()
        .filter(Some("sqpack"), log::LevelFilter::Trace)
        .try_init();

    #[cfg(windows)]
    let pack = SqPackPackage::new(Path::new("D:\\Games\\FINAL FANTASY XIV - KOREA\\game\\sqpack"))?;
    #[cfg(unix)]
    let pack = SqPackPackage::new(Path::new("/mnt/d/Games/FINAL FANTASY XIV - KOREA/game/sqpack"))?;

    {
        let data = pack.read_file("exd/classjob.exh").await?;
        assert_eq!(data[0], b'E');
        assert_eq!(data[1], b'X');
        assert_eq!(data[2], b'H');
        assert_eq!(data[3], b'F');
        assert_eq!(data.len(), 238);
    }

    {
        let data = pack.read_file("bg/ex1/01_roc_r2/common/bgparts/r200_a0_bari1.mdl").await?;
        assert_eq!(data[0], 3u8);
        assert_eq!(data.len(), 185_088);
    }

    {
        let data = pack.read_file("common/graphics/texture/dummy.tex").await?;
        assert_eq!(data[0], 0u8);
        assert_eq!(data[1], 0u8);
        assert_eq!(data[2], 128u8);
        assert_eq!(data[3], 0u8);
        assert_eq!(data.len(), 104);
    }

    {
        let data = pack.read_file("chara/equipment/e6016/texture/v01_c0201e6016_met_m.tex").await?;
        assert_eq!(data[0], 0u8);
        assert_eq!(data[1], 0u8);
        assert_eq!(data[2], 128u8);
        assert_eq!(data[3], 0u8);
        assert_eq!(data.len(), 43784);
    }

    Ok(())
}
