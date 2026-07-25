#!/usr/bin/env python

import arcadia

def report(uni):
    print(f'At tick {uni.timetick}, billing {uni.billing}')
    for w in uni.worlds:
        print(f'World #{w.id}, production {w.production}')
    for a in uni.actors:
        print(f'Actor #{a.id}, credits {a.credits} at #{a.home.id}, birth {a.control.threshold} at {a.control.giveaway}')

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Arcadia Universe status report")
    parser.add_argument("filename", type=str, help="File to load")
    args = parser.parse_args()

    with open(args.filename, 'rb') as f:
        uni = arcadia.load(f)
        report(uni)

if __name__ == "__main__":
    main()
