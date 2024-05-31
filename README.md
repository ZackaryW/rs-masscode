# rs-masscode
rust written middleware for masscode

## Install
```bash
cargo install rs-masscode
```
## Features
- query items using v8_mini engine
- subqueries and return only ids
- add new entries (WIP)

> WIP - work in progress

## Usage
### Binary Examples
```bash
rsmasscode query folder folder -i
>>> ["pIb-o2Pf","aY2u3l_e","94PU-wE5","TFxS73oz","hqZ54bAI","Gyi3iucF","QijTSXIj","ZGv04n4k","jI0pbDIh","oPxez4ew"]
rsmasscode query tag "tag.name.startsWith('s')"
>>>[
>>>  {
>>>    "name": "swing",
>>>    "id": "812HVnMd",
>>>    "createdAt": 1660069353413,
>>>    "updatedAt": 1660069353413
>>>  }
>>>]
rsmasscode query snippet "snippet.name.startsWith('mul')" -i --tag "tag.name.endsWith('g')"
["hM6sINGJ"]
```

## Additional Info
[repo link](https://github.com/ZackaryW/rs-masscode)
