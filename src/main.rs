extern crate nbt;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::io;
use std::path::PathBuf;

use serde_json::Value;

/** Create an InvalidData io::Error with the description being a
 * formatted string */
macro_rules! io_error {
    ($fmtstr:tt) => { io_error!($fmtstr,) };
    ($fmtstr:tt, $( $args:expr ),* ) => {
        Err(::std::io::Error::new(::std::io::ErrorKind::Other,
                                  format!($fmtstr, $( $args ),* )));
    };
}

const AMOUNT: usize = 300;

fn main() {
    let files = get_stats_files("./stats").unwrap();

    let mut stats = Vec::with_capacity(files.len());

    for file in files {
        stats.push(read_stats(&file).expect(&format!("Error reading stats from {:?}", &file)));
    }

    stats.sort();
    let stats: Vec<StatsFile> = stats.drain(..).take(AMOUNT).collect();

    /* The static numbers column commented out because it doesn't align properly
     * https://meta.wikimedia.org/wiki/Help:Sorting#Static_column
    println!(r#"{{|"#);
    println!("|-");
    println!("|");
    println!(r#"{{| class="wikitable" style="margin-right:0""#);
    println!("! Number");
    for i in 1..AMOUNT+1 {
        println!("|-");
        println!("! {}", i);
    }
    println!("|}}");
    println!("|");
    */

    println!(r#"{{| class="wikitable sortable" style="margin-left:0""#);
    println!("|-");
    println!(r#"! Player !! Play time (hours) !! Games quit !! Jumps !! Deaths !! Damage taken (half hearts) !! Damage dealt (half hearts) !! Mob kills !! Player kills !! Traveled (km) !! Cake slices eaten !!data-sort-type="number" | Advancements"#);
    for stat in stats {
        println!("|-");
        println!("{}", &stat);
    }
    println!("|}}");
}

/// Returns a list of paths to each of the json files in the stats directory
fn get_stats_files(dir: &str) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(dir)?
       .map(|x| x.unwrap().path().to_path_buf())
       .filter(|x| {
           match x.file_name() {
               /* json file names have exactly 41 characters,
                * not super important that we throw away non-json names now,
                * but no reason not to */
               Some(name) if name.len() == 41 => (),
               _ => return false,
           }
           match x.extension() {
               Some(extension) if extension == "json" => true,
               _ => false,
           }
       })
       .collect())
}


macro_rules! stats_file {
    (
        $( $name:ident, $typ:ty, $default:expr );+
    ) => {
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[allow(non_snake_case)]
        struct StatsFile {
            $(
                $name: $typ
            ),+
        }
        impl StatsFile {
            fn none_to_default(&mut self) {
                $(
                    if let None = self.$name {
                        self.$name = $default;
                    }
                )+
            }
        }

    };
}
stats_file!(playername, Option<String>, Some("Unknown name".to_string());
            stat_playOneMinute, Option<u64>, Some(0);
            stat_leaveGame, Option<u64>, Some(0);
            stat_jump, Option<u64>, Some(0);
            stat_deaths, Option<u64>, Some(0);
            stat_damageTaken, Option<u64>, Some(0);
            stat_damageDealt, Option<u64>, Some(0);
            stat_mobKills, Option<u64>, Some(0);
            stat_playerKills, Option<u64>, Some(0);
            stat_walkOneCm, Option<u64>, Some(0);
            stat_crouchOneCm, Option<u64>, Some(0);
            stat_sprintOneCm, Option<u64>, Some(0);
            stat_swimOneCm, Option<u64>, Some(0);
            stat_fallOneCm, Option<u64>, Some(0);
            stat_climbOneCm, Option<u64>, Some(0);
            stat_flyOneCm, Option<u64>, Some(0);
            stat_diveOneCm, Option<u64>, Some(0);
            stat_minecartOneCm, Option<u64>, Some(0);
            stat_boatOneCm, Option<u64>, Some(0);
            stat_pigOneCm, Option<u64>, Some(0);
            stat_horseOneCm, Option<u64>, Some(0);
            stat_aviateOneCm, Option<u64>, Some(0);
            distanceMeter, Option<u64>, Some(0);
            stat_cakeSlicesEaten, Option<u64>, Some(0);
            advancements_count, Option<u64>, Some(0));

impl Ord for StatsFile {
    fn cmp(&self, other: &StatsFile) -> Ordering {
        self.stat_playOneMinute.cmp(&other.stat_playOneMinute).reverse()
    }
}
impl PartialOrd for StatsFile {
    fn partial_cmp(&self, other: &StatsFile) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl fmt::Display for StatsFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "| [[{playername}]] || {playtime} || {leavegame} || {jump} || {deaths} || {taken} || {dealt} || {mobkills} || {playerkills} || {distance} || {cakeslices} || {advancements}/55",
               playername=self.playername.clone().unwrap(),
               playtime=self.stat_playOneMinute.unwrap() / (20 * 60 * 60),
               leavegame=self.stat_leaveGame.unwrap(),
               jump=self.stat_jump.unwrap(),
               deaths=self.stat_deaths.unwrap(),
               taken=self.stat_damageTaken.unwrap() / 10,
               dealt=self.stat_damageDealt.unwrap() / 10,
               mobkills=self.stat_mobKills.unwrap(),
               playerkills=self.stat_playerKills.unwrap(),
               distance=self.distanceMeter.unwrap(),
               cakeslices=self.stat_cakeSlicesEaten.unwrap(),
               advancements=self.advancements_count.unwrap())
    }
}


/// Read a single stats file
fn read_stats(path: &PathBuf) -> io::Result<StatsFile> {
    let uuid = match path.file_stem() {
        Some(x) => x.to_str().expect("Invalid player uuid").to_string(),
        None => return io_error!("File name did not contain a uuid"),
    };

    let mut f = File::open(path)?;
    let mut tmp = String::new();
    f.read_to_string(&mut tmp)?;
    /* Use underscores instead of . so that we can take advantage of
     * RustcDecodable, since Rust doesn't allow the use of . */
    tmp = tmp.replace(".", "_");
    let mut ret: StatsFile = serde_json::from_str(&tmp)?;
    ret.none_to_default();

    ret.distanceMeter = Some((ret.stat_walkOneCm.unwrap()
        + ret.stat_crouchOneCm.unwrap()
        + ret.stat_sprintOneCm.unwrap()
        + ret.stat_swimOneCm.unwrap()
        + ret.stat_fallOneCm.unwrap()
        + ret.stat_climbOneCm.unwrap()
        + ret.stat_flyOneCm.unwrap()
        + ret.stat_diveOneCm.unwrap()
        + ret.stat_minecartOneCm.unwrap()
        + ret.stat_boatOneCm.unwrap()
        + ret.stat_pigOneCm.unwrap()
        + ret.stat_horseOneCm.unwrap()
        + ret.stat_aviateOneCm.unwrap()) / (100 * 1000));

    let advancements_path = {
        let mut path = path.clone();
        assert!(path.pop()); /* assert so that we unwind if path is empty */
        assert!(path.pop()); /* and we pop twice to remove both uuid.json and stats */
        path.push("advancements");
        path.push(format!("{}.json", uuid));
        path
    };
    ret.advancements_count = Some(count_advancements(&advancements_path).unwrap());

    /* It seems a few players don't have a uuid playerfile, these players are
     * unimportant so we just ignore them */
    if let Ok(name) = get_player_name(&uuid) {
        ret.playername = Some(name);
    } else {
        ret.playername = Some(uuid.clone());
    }

    Ok(ret)
}

/// Read the given advancements file, returning the amount of gained achievements
fn count_advancements(path: &PathBuf) -> io::Result<u64> {

    let mut f = match File::open(path) {
        Ok(f) => f,
        /* If file is not found, the player has 0 advancements.
         * This happens when the player has not logged in since advancements
         * were added to the game */
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(0);
        },
        Err(e) => {
            panic!(e);
        },
    };
    let mut tmp = String::new();
    f.read_to_string(&mut tmp)?;
    let json: Value = serde_json::from_str(&tmp)?;
    let json = json.as_object().unwrap();
    /* json is now a map with the keys being the names of the advancements,
     * and the values being a json object of details over the advancement */

    let mut count = 0;

    for (name, value) in json {
        if name.starts_with("minecraft:story") ||
            name.starts_with("minecraft:nether") ||
            name.starts_with("minecraft:end") ||
            name.starts_with("minecraft:adventure") ||
            name.starts_with("minecraft:husbandry")
        { }
        else {
            continue;
        }

        match value.get("done") {
            Some(x) if x.is_boolean() => {
                if x.as_bool().expect("wasn't really bool") {
                    count += 1;
                }
            },
            Some(_) => (),
            _ => (),
        }
    }

    Ok(count)
}


/// Given a UUID find the player name of the UUID from the playerdata dir
fn get_player_name(uuid: &str) -> io::Result<String> {
    let mut f = File::open(format!("./playerdata/{}.dat", uuid))?;
    let nbt = nbt::Blob::from_gzip(&mut f)?;

    let nbt = match nbt["bukkit"] {
        nbt::Value::Compound(ref x) => x.clone(),
        _ => return io_error!("Could not find bukkit compound in NBT"),
    };
    let name = match nbt.get("lastKnownName") {
        Some(x) => x.clone(),
        None => return io_error!("lastKnownName not found in NBT"),
    };
    let name = match name {
        nbt::Value::String(ref x) => x.clone(),
        _ => return io_error!("lastKnownName had invalid type in NBT"),
    };

    Ok(name)
}

