
We have multiple kinds of LogLines:
```log
2024/12/12 20:53:08 181924187 23ef79e1 [DEBUG Client 30776] [EOS SDK] LogEOSAuth: Sending Verify Auth request. url=<Redacted>
2024/12/12 20:53:09 181924984 23ef79e1 [DEBUG Client 30776] [EOS SDK] LogEOSAuth: Verify Auth Success
2024/12/12 20:53:27 181942859 2d8e8abd [DEBUG Client 30776] Got Instance Details from login server
2024/12/12 20:53:27 181942875 403248f7 [INFO Client 30776] [SHADER] Delay: OFF
2024/12/12 20:53:27 181942875 91c6da6 [INFO Client 30776] Connecting to instance server at 103.1.214.136:21360
2024/12/12 20:53:27 181942906 91c69aa [DEBUG Client 30776] Connect time to instance server was 15ms
2024/12/12 20:53:27 181942937 91c4c6b [DEBUG Client 30776] Client-Safe Instance ID = 1640157749
2024/12/12 20:53:27 181942937 2caa1679 [DEBUG Client 30776] Generating level 32 area "G2_town" with seed 1
2024/12/12 20:53:30 181946203 f0c29e12 [INFO Client 30776] Tile hash: 1414081272
2024/12/12 20:53:30 181946203 f0c29e11 [INFO Client 30776] Doodad hash: 1797091165
2024/12/12 20:53:31 181946281 4228e34f [DEBUG Client 30776] [ENTITY] Destroy static entities
2024/12/12 20:53:31 181946312 4228e280 [DEBUG Client 30776] [ENTITY] Finalize static entities
2024/12/12 20:53:31 181946312 4228e34f [DEBUG Client 30776] [ENTITY] Destroy static entities
2024/12/12 20:53:31 181946328 992412d1 [DEBUG Client 30776] [SCENE] Height Map Texture: 540 x 480
2024/12/12 20:53:31 181946343 4228e280 [DEBUG Client 30776] [ENTITY] Finalize static entities
2024/12/12 20:53:31 181946437 1a61ee40 [DEBUG Client 30776] Joined guild named Hairy_Dad_Bods with 2 members 
2024/12/12 20:53:31 181946578 1a61ea26 [DEBUG Client 30776] InstanceClientSetSelfPartyInvitationSecurityCode = 0
2024/12/12 20:53:31 181946781 833064f7 [CRIT Client 30776] File Not Found: Metadata/Effects/Microtransactions/Flasks/LightningInABottle/EPKs/characterFX.epk. Using fallback EPK file
2024/12/12 20:53:32 181947578 ff73624f [CRIT Client 30776] No collision callback set! MoveTo was not called before setting up line sweeper. Metadata/Characters/DexInt/DexIntFourb
2024/12/12 20:53:35 181950500 403248f7 [INFO Client 30776] [SHADER] Delay: ON
2024/12/12 20:53:37 181952906 3ef2336d [INFO Client 30776] : 3 Items identified
2024/12/12 20:54:08 181983500 3ef2336d [INFO Client 30776] : Trade accepted.
2024/12/27 20:23:31 94685156 4d32db07 [WARN Client 29384] Tried to SetExtraActivationRangeAroundTile() after level generation was complete
```

First we have LogLevel:
```rust
enum PoeLogLvl{
    Info,
    Critical,
    Debug,
    Warn,
}
```
which we need to parse out of ```[INFO Client 30776]```
the `Client \d` is a .client_number. 


the format of a log message, best parsed:
```
2024/12/12 20:54:08 181983500 3ef2336d [INFO Client 30776] : Trade accepted.

```

There is a case where the []s appear twice i.e :
```
2024/12/12 20:53:31 181946281 4228e34f [DEBUG Client 30776] [ENTITY] Destroy static entities
```
so it seems we should be checking to see if our LogLine is 'nested' where nested would be indicated by greater than a single set of []s.

if we are nested, we just have a 'name' of the 2nd level of nesting which we'll do with a String.

A log indicating a Chat::Whisper is in this format:
`2024/12/26 08:57:55 6750687 3ef2336f [INFO Client 25112] @To blka: ahh so sorry i made mistake on currency and don't have. my bad.
`

so it'll alawys begin with the @To $player_name syntax.

These logs are from engine about gameplay:
```log
2024/12/26 09:08:47 7402765 3ef2336f [INFO Client 25112] : 9 Items identified
2024/12/26 09:09:18 7434046 3ef2336f [INFO Client 25112] : Trade accepted.
```

logs like this indicate a player has been slain 
`2024/12/26 09:27:41 8537562 3ef2336f [INFO Client 25112] : jengablox has been slain.`

when initially parsing the log for the first time we should keep a globally available DEATHS hashmap of <PlayerName, DeathCount> as <String, usize>

the anatomy of a log is:
`2024/12/26 09:58:47 10403343 3ef2336f [INFO Client 25112] : 2 Items identified`
`$yyyy/mm/dd $hh:mm:ss $micros $log_hash $[log lvl $client_id] $msg`
- again, any msg beginning `: $msg` is from engine about gameplay.

the logs have a special line:
```2024/12/27 08:52:21 ***** LOG FILE OPENING *****```
which indicates the BEGINNING of a $session.

We should add a .sessions field to our Config that is an enum as follows:
```rust
enum Session{
    All,
    Latest,
    Recent(usize)
}
```
we should keep in memory the sessions i,e ALL or only the most recent or only the most recent(n) per this config option.

In an example like this:
```log
2025/01/02 09:52:55 11546218 3ef2336f [INFO Client 25512] : jengablox (Invoker) is now level 54
2025/01/02 09:53:03 11553812 3ef2336f [INFO Client 25512] @To SlightOfFist: Gale Rosary, Lunar Amulet
2025/01/02 09:53:09 11560406 3ef2336f [INFO Client 25512] @To SlightOfFist: Corpse Pendant, Gold Amulet
2025/01/02 09:53:21 11571765 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Sending Verify Auth request. url=<Redacted>
2025/01/02 09:53:21 11572046 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Verify Auth Success
2025/01/02 09:54:21 11632046 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Sending Verify Auth request. url=<Redacted>
2025/01/02 09:54:22 11632828 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Verify Auth Success
2025/01/02 09:54:37 11648187 3ef2336f [INFO Client 25512] : Trade accepted.
2025/01/02 09:55:22 11692828 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Sending Verify Auth request. url=<Redacted>
2025/01/02 09:55:22 11693109 23ef79e1 [DEBUG Client 25512] [EOS SDK] LogEOSAuth: Verify Auth Success
2025/01/02 09:55:53 11723656 3ef2336f [INFO Client 25512] : SlightOfFist has left the area.
2025/01/02 09:55:53 11723656 3ef2336f [INFO Client 25512] : Trade cancelled.
2025/01/02 09:56:08 11738843 f4ab40f7 [INFO Client 25512] Successfully allocated passive skill id: elemental20, name: Elemental
```
we can see that, $player_name == jengablox leveld up, we should assume that the most recently available one of these is our player's $CURRENT_LEVEL, which is one of a few constants I want us to keep track of including $CURRENT_SESSION_LENGTH which should be a value of micros from the begininng of the current session to the most recent of LogLine's timestamps.

`2025/01/02 10:03:58 12209531 3ef2336f [INFO Client 25512] : SlightOfFist has joined the area.`
this indicated that someone on our current $PARTY is in the same area of us.

we should refactor our code to incorporate all my ideas here.

When running in stdout for example, we should print something like this:
```
$most recent log line -4
$most recent log line -3 
$most recent log line -2
$most recent log line -1
$most recent log line

PLAYER_LVL = $PLAYER_LEVEL
PLAYING_AS = $PLAYER_NAME
CURRENT_SESSION = hh:mm:ss
Started at = StartTime of THIS APP
```
When we print the logline, we can omit the date, just use the HH:MM:SS 
We can also omit the hash, and the `[INFO Client 25512]` etc.

our toml looks like this:
```toml
# // Example .toml file for the configuration
player_names = ["jengablox", "tittyBlaster"]
client_log_location = "Client.txt"
always_include = [] # renamed from 'filters'
always_exclude = ["EOS SDK", "VULKAN", "ENTITY", "RENDER", "SHADER", "SCENE", "DEBUG", "SOUND"]
```
we can use the player names to help us sort the levels properly (unfortunately all games write to the same log, regardless of which palyer character you may be using -- we don't wanna spoil the data.)

We should be doing a union of the include/exclude items so that anything in the always_include overrides (by deleting) anything in the exclude list.



