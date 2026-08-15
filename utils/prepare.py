#!/usr/bin/env python

import arcadia

def make():
    uni = arcadia.Universe()
    uni.billing = 10
    world = uni.addworld()
    world.production = 100
    actor = uni.addactor(world)
    actor.credits = 1000
    actor.control.creditsensor.precision = 10
    actor.control.birthsignal.scale = 800.0
    actor.control.birthsignal.threshold = -1.0
    actor.control.birthsignal.variation = 300
    actor.control.birthcredit.giveaway = 400
    actor.control.values.credits = 1000.0
    actor.control.values.birth = False
    actor.control.threshold = 750
    actor.control.giveaway = 400
    world.actors.append(actor)
    return uni

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Produces Arcadia Universe")
    parser.add_argument("filename", type=str, help="File to create")
    args = parser.parse_args()

    with open(args.filename, 'wb') as f:
        make().save(f)

if __name__ == "__main__":
    main()
