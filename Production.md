# Production Record
Here is the latest production record of each disc.

Last updated: 2017-04-23T02:30

|Date		|Disc	|Serial	|Checksum|
|---		|---	|---	|---	|
|2017-04-19	|Disc-1	|0	|/disc-1.md5 Checksum Error (Bug #1)|
|2017-04-19	|Disc-1	|1	|/disc-1.md5 Checksum Error (Bug #1)|
|2017-04-19	|Disc-2	|0	|Success|
|2017-04-20	|Disc-1	|2	|Success|
|2017-04-20	|Disc-3	|0	|Success|
|2017-04-20	|Disc-4	|0	|Success|
|2017-04-20	|Disc-2	|1	|Success|
|2017-04-20	|Disc-5	|0	|Success|
|2017-04-21	|Disc-3	|1	|Success|
|2017-04-21	|Disc-6	|0	|Success|
|2017-04-21	|Disc-4	|1	|Success|
|2017-04-21	|Disc-7	|0	|Success|
|2017-04-22	|Disc-8	|0	|Success|
|2017-04-22	|Disc-9	|0	|Success|
|2017-04-22	|Disc-9	|1	|Success|
|2017-04-22	|Disc-8	|1	|Success|
|2017-04-22	|Disc-7	|1	|Not checked|
|2017-04-22	|Disc-7	|2	|Not checked|
|2017-04-22	|Disc-6	|1	|Not checked|
|2017-04-22	|Disc-5	|1	|Not checked|

# Serial Number List
This is the list of the disc serial number of the last disc produced.

Last updated: 2017-04-23T02:30

|Disc	|Serial	|Date	|
|---	|---	|---	|
|Disc-1	|2	|2017-04-20|
|Disc-2	|1	|2017-04-20|
|Disc-3	|1	|2017-04-21|
|Disc-4	|1	|2017-04-21|
|Disc-5	|1	|2017-04-22|
|Disc-6	|1	|2017-04-22|
|Disc-7	|2	|2017-04-22|
|Disc-8	|1	|2017-04-22|
|Disc-9	|1	|2017-04-22|

## Known bugs and Notes

### Bug 1
Disc-1 Serial 0 and 1 were all known to have the md5sum of the md5sum file itself written into the file.
This would cause the direct md5sum check of the disc-1.md5 file to fail. Using the md5sum provided by
this github page would work.
