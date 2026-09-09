# Copycatch

## Projet Description

Copycatch is a command-line-tool for managing and creating backups of photo libraries. Copycatch analyzes your photos locally during the backup process and identifies duplicate images which may be bloating your library!

## Motivation

Photos often capture valuable moments in our lives that we wish to hold on to and cherish forever. Two major challenges arise when dealing with photos in all forms

- Where should I keep my photos (so I can find them later) ?
- How can I preserve my photos for the future?

While attempting to answer these questions myself, I realized there are many tools available for managing photos, but none of them were quite right for my needs. Modern cloud-based photo managers can be incredibly convenient to use, but they often come with *privacy concerns* and *costly subscription fees* which tend to scale with the size of the photo library. 

This pushed me to find a solution which could provide better quality of life features for managing photos locally. Features such as duplicate detection and image similarity ratings help ensure that valuable photos are saved while helping users avoid wasting storage space on duplicates. Copycatch isn't designed to *replace* cloud storage solutions, in fact, a leaner photo library can *better* leverage the redundancy of cloud storage to provide a more robust backup solution while keeping costs at a minimum.

## Quick Start

 1.) Install Rust from the [Official Website](https://rust-lang.org/tools/install/) on your local machine.
    

 2.) Install Copycatch using Cargo

```bash
cargo install --git https://github.com/ClassicMike48/Copycatch
```

 3.) Start building your library

 ```bash
copycatch --source /path/to/source --destination /path/to/destination
```
or 
```bash
copycatch -s /path/to/source -d /path/to/destination
```

## Building the project

1.) Clone the repository
```bash
git clone https://github.com/ClassicMike48/Copycatch
```

2.) Move into the project directory 
```bash
cd Copycatch
```

3.) Run the project in dev mode
```bash
cargo run 
```