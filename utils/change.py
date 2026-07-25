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

    parser = argparse.ArgumentParser(description="Arcadia Universe database modification")
    parser.add_argument("filename", type=str, help="File to load")
    parser.add_argument("expr", type=str, help="Expression")
    args = parser.parse_args()
    with open(args.filename, 'rb') as f:
        uni = arcadia.load(f)
    eval(args.expr, {'uni':uni})
    report(uni)
    with open(args.filename, 'wb') as f:
        uni.save(f)

if __name__ == "__main__":
    main()
