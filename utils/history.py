#!/usr/bin/env python

import arcadia

def report(history):
    for msg in history.messages:
        print(f'{msg.timetick:08}: {msg.payload}')

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Arcadia Universe history")
    parser.add_argument("filename", type=str, help="History file")
    args = parser.parse_args()

    with open(args.filename, 'rb') as f:
        history = arcadia.load_history(f)
        report(history)

if __name__ == "__main__":
    main()
