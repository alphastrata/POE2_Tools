/*  What is this module all about?


GRANULARITY = u64:
        BFS shortest path       time:   [8.0797 ms 8.1029 ms 8.1320 ms]
                                change: [+2.9860% +3.3056% +3.6995%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 5 outliers among 100 measurements (5.00%)
        3 (3.00%) high mild
        2 (2.00%) high severe

        Dijkstra shortest path  time:   [8.1074 ms 8.1206 ms 8.1373 ms]
                                change: [+4.0406% +4.2263% +4.4625%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 17 outliers among 100 measurements (17.00%)
        1 (1.00%) low mild
        16 (16.00%) high severe

        BFS shortest path reversed
                                time:   [6.5755 ms 6.5838 ms 6.5928 ms]
                                change: [+3.9146% +4.0483% +4.1868%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 3 outliers among 100 measurements (3.00%)
        3 (3.00%) high mild

        Dijkstra shortest path reversed
                                time:   [6.4180 ms 6.4293 ms 6.4435 ms]
                                change: [+2.8799% +3.0919% +3.3513%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 6 outliers among 100 measurements (6.00%)
        2 (2.00%) high mild
        4 (4.00%) high severe

            Running benches\str_vs_enum_match.rs (target\release\deps\str_vs_enum_match-6cc64e43f49f083a.exe)
        Gnuplot not found, using plotters backend
        string match filter     time:   [234.14 µs 240.63 µs 248.53 µs]
                                change: [+2.6070% +4.2736% +6.4978%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 16 outliers among 100 measurements (16.00%)
        4 (4.00%) high mild
        12 (12.00%) high severe

        type match filter       time:   [111.09 µs 112.20 µs 113.41 µs]
                                change: [-0.4484% +0.1165% +0.7371%] (p = 0.71 > 0.05)
                                No change in performance detected.
        Found 12 outliers among 100 measurements (12.00%)
        3 (3.00%) high mild
        9 (9.00%) high severe

            Running benches\take_while.rs (target\release\deps\take_while-2c0fa3413e09d31a.exe)
        Gnuplot not found, using plotters backend
        take_while_contains_MeleeDamage
                                time:   [15.426 ms 15.735 ms 16.092 ms]
                                change: [+14.787% +16.950% +19.751%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 14 outliers among 100 measurements (14.00%)
        3 (3.00%) high mild
        11 (11.00%) high severe

        par_take_while_contains_MeleeDamage
                                time:   [27.453 ms 27.517 ms 27.586 ms]
                                change: [+7.9863% +8.3564% +8.7081%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 3 outliers among 100 measurements (3.00%)
        2 (2.00%) high mild
        1 (1.00%) high severe

        par_take_while_many_selections
                                time:   [27.981 ms 28.031 ms 28.082 ms]
                                change: [+10.432% +10.783% +11.108%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 1 outliers among 100 measurements (1.00%)
        1 (1.00%) high mild

        take_while_many_selections
                                time:   [14.664 ms 14.700 ms 14.739 ms]
                                change: [+9.1420% +9.4770% +9.8273%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 7 outliers among 100 measurements (7.00%)
        6 (6.00%) high mild
        1 (1.00%) high severe

GRANULARITY = u32:
        BFS shortest path       time:   [7.8237 ms 7.8462 ms 7.8758 ms]
                                change: [-3.5933% -3.1677% -2.6764%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 3 outliers among 100 measurements (3.00%)
        3 (3.00%) high severe

        Dijkstra shortest path  time:   [8.2572 ms 8.2690 ms 8.2839 ms]
                                change: [+1.5732% +1.8271% +2.0759%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 3 outliers among 100 measurements (3.00%)
        1 (1.00%) high mild
        2 (2.00%) high severe

        BFS shortest path reversed
                                time:   [6.3669 ms 6.3770 ms 6.3908 ms]
                                change: [-3.3367% -3.1417% -2.8817%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 13 outliers among 100 measurements (13.00%)
        12 (12.00%) high mild
        1 (1.00%) high severe

        Dijkstra shortest path reversed
                                time:   [6.4381 ms 6.4440 ms 6.4503 ms]
                                change: [-0.0102% +0.2281% +0.4364%] (p = 0.04 < 0.05)
                                Change within noise threshold.
        Found 7 outliers among 100 measurements (7.00%)
        6 (6.00%) high mild
        1 (1.00%) high severe

            Running benches\str_vs_enum_match.rs (target\release\deps\str_vs_enum_match-6cc64e43f49f083a.exe)
        Gnuplot not found, using plotters backend
        string match filter     time:   [220.65 µs 220.78 µs 220.91 µs]
                                change: [-7.3963% -5.4748% -3.8883%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 7 outliers among 100 measurements (7.00%)
        2 (2.00%) high mild
        5 (5.00%) high severe

        type match filter       time:   [109.31 µs 109.47 µs 109.63 µs]
                                change: [-2.1456% -1.6178% -1.1561%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 5 outliers among 100 measurements (5.00%)
        2 (2.00%) high mild
        3 (3.00%) high severe

            Running benches\take_while.rs (target\release\deps\take_while-2c0fa3413e09d31a.exe)
        Gnuplot not found, using plotters backend
        take_while_contains_MeleeDamage
                                time:   [13.703 ms 13.731 ms 13.764 ms]
                                change: [-14.750% -12.738% -10.990%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 13 outliers among 100 measurements (13.00%)
        2 (2.00%) low mild
        5 (5.00%) high mild
        6 (6.00%) high severe

        par_take_while_contains_MeleeDamage
                                time:   [24.992 ms 25.070 ms 25.151 ms]
                                change: [-9.2409% -8.8947% -8.5128%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 2 outliers among 100 measurements (2.00%)
        1 (1.00%) high mild
        1 (1.00%) high severe

        par_take_while_many_selections
                                time:   [26.316 ms 26.356 ms 26.396 ms]
                                change: [-6.1966% -5.9766% -5.7611%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 1 outliers among 100 measurements (1.00%)
        1 (1.00%) high mild

        take_while_many_selections
                                time:   [13.615 ms 13.640 ms 13.672 ms]
                                change: [-7.5062% -7.2065% -6.8974%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 15 outliers among 100 measurements (15.00%)
        1 (1.00%) low mild
        8 (8.00%) high mild
        6 (6.00%) high severe

GRANULARITY = u16:
        BFS shortest path       time:   [7.6462 ms 7.6640 ms 7.6844 ms]
                                change: [-2.7481% -2.3223% -1.9037%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 2 outliers among 100 measurements (2.00%)
        1 (1.00%) high mild
        1 (1.00%) high severe

        Dijkstra shortest path  time:   [7.8631 ms 7.8716 ms 7.8838 ms]
                                change: [-5.0150% -4.8062% -4.6235%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 9 outliers among 100 measurements (9.00%)
        2 (2.00%) low mild
        3 (3.00%) high mild
        4 (4.00%) high severe

        BFS shortest path reversed
                                time:   [6.2709 ms 6.2822 ms 6.2943 ms]
                                change: [-1.7647% -1.4861% -1.2413%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 1 outliers among 100 measurements (1.00%)
        1 (1.00%) high mild

        Dijkstra shortest path reversed
                                time:   [6.1902 ms 6.2023 ms 6.2164 ms]
                                change: [-3.9614% -3.7504% -3.5301%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 13 outliers among 100 measurements (13.00%)
        2 (2.00%) low mild
        2 (2.00%) high mild
        9 (9.00%) high severe

            Running benches\str_vs_enum_match.rs (target\release\deps\str_vs_enum_match-6cc64e43f49f083a.exe)
        Gnuplot not found, using plotters backend
        string match filter     time:   [224.38 µs 224.60 µs 224.84 µs]
                                change: [+1.5675% +1.8470% +2.0622%] (p = 0.00 < 0.05)
                                Performance has regressed.
        Found 25 outliers among 100 measurements (25.00%)
        14 (14.00%) low mild
        7 (7.00%) high mild
        4 (4.00%) high severe

        type match filter       time:   [110.40 µs 110.52 µs 110.67 µs]
                                change: [+0.8358% +1.1264% +1.3728%] (p = 0.00 < 0.05)
                                Change within noise threshold.
        Found 13 outliers among 100 measurements (13.00%)
        1 (1.00%) high mild
        12 (12.00%) high severe

            Running benches\take_while.rs (target\release\deps\take_while-2c0fa3413e09d31a.exe)
        Gnuplot not found, using plotters backend
        take_while_contains_MeleeDamage
                                time:   [13.443 ms 13.464 ms 13.488 ms]
                                change: [-2.2070% -1.9394% -1.6805%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 1 outliers among 100 measurements (1.00%)
        1 (1.00%) high severe

        par_take_while_contains_MeleeDamage
                                time:   [25.330 ms 25.411 ms 25.503 ms]
                                change: [+0.9071% +1.3627% +1.8718%] (p = 0.00 < 0.05)
                                Change within noise threshold.
        Found 4 outliers among 100 measurements (4.00%)
        2 (2.00%) high mild
        2 (2.00%) high severe

        par_take_while_many_selections
                                time:   [25.547 ms 25.604 ms 25.662 ms]
                                change: [-3.1257% -2.8531% -2.5701%] (p = 0.00 < 0.05)
                                Performance has improved.
        Found 2 outliers among 100 measurements (2.00%)
        2 (2.00%) high mild

        take_while_many_selections
                                time:   [13.451 ms 13.471 ms 13.495 ms]
                                change: [-1.5154% -1.2430% -0.9756%] (p = 0.00 < 0.05)
                                Change within noise threshold.
        Found 14 outliers among 100 measurements (14.00%)
        5 (5.00%) high mild
        9 (9.00%) high severe
*/

//NOTE: the findings above basically indicate we get a +5 to +15% peroformance boost, on our MOST called
// functionality i.e take_while and bfs from using a smaller value for our 'granularity'

pub type GRANULATITY = u16; // The smallest size we can fit all our unique NODE_IDs into.
pub type GroupId = GRANULATITY;
pub type NodeId = GRANULATITY;
pub type EdgeId = GRANULATITY;
