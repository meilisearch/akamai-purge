use structopt::StructOpt;

use akamai::purge_tag;

#[derive(Debug, StructOpt)]
struct Opt {
    tags_to_purge: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    match purge_tag(opt.tags_to_purge.clone()) {
        Ok(_) => println!("Tags {:?} purged !", opt.tags_to_purge),
        Err(err) => eprintln!("Impossible to purge cache tagged as {:?}; {}", opt.tags_to_purge, err)
    }
}
