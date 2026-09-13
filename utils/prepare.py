#!/usr/bin/env python

import pathlib
import yaml
import arcadia

def make(args):
    uni = arcadia.Universe()
    uni.billing = 10
    world = uni.addworld()
    world.production = 1000
    actor = uni.addactor(world)
    actor.credits = 1000
    with open(args.template) as tfile:
        ytmpl = yaml.load(tfile, Loader=yaml.Loader)
        for junit in ytmpl['control']['units']:
            actor.control.units.append( uni.units.make_unit(junit) )
        for jvalue in ytmpl['control']['values']:
            actor.control.values.values.append( uni.units.make_value(jvalue) )
    actor.control.values.credits = 1000.0
    actor.control.values.birth = False
    world.actors.append(actor)
    return uni

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Produces Arcadia Universe")
    parser.add_argument("filename", type=str, help="File to create")
    parser.add_argument("template", type=str, help="Template to create from")
    args = parser.parse_args()

    with open(args.filename, 'wb') as f:
        make(args).save(f)
    pathlib.Path(args.filename).with_suffix('.history').unlink(missing_ok=True)

if __name__ == "__main__":
    main()
