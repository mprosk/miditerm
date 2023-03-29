import csv
import json

with open('midi_sysex_ids.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    jsondata = []
    for row in reader:
        mfgid = []
        for byte in row['id'].split('.'):
            try:
                mfgid.append(int(byte, 16))
            except ValueError as e:
                print(row)
                raise e
        entry = {
            'id': mfgid[:],
            'manufacturer': row['manufacturer'],
            'group': row['group'],
            'reserved': bool(row['reserved'])
        }
        if row['status']:
            entry['status'] = row['status']
        jsondata.append(entry)

    with open('ids.json', mode='w') as jsonfile:
        json.dump(jsondata, jsonfile, ensure_ascii=True, indent=2)
