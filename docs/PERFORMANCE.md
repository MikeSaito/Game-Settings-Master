# Performance and concurrency audit

The audit compares v1.5.3 with the current branch using optimized Windows
`native-test` builds. Test data, update endpoints, profiles, and config writes
are isolated from real installations. It does not launch or benchmark a game.

Run **Windows desktop E2E** manually on the branch with `test_updates=false`
and `test_performance=true`. The performance job builds both revisions and
runs the same workload sequentially on one Windows Server 2022 runner.

The workload contains eight manual game profiles, eight diagnostic baselines
with 200 operations each, 1,000-line Engine.ini files, and 256 completed
transaction journals. It records three startup attempts, waits 30 seconds,
then samples the library, minimized application, editor, and editor after
20 preview/cancel cycles for 60 seconds per phase.

The process sampler includes the application and its WebView2 descendants.
CPU percentages are normalized to all logical processors. Private bytes are
committed private memory, not resident RAM. Summed working sets can count shared
pages more than once and must not be presented as unique physical memory.
Handle, thread, and process counts are retained in the JSON samples.

First contentful paint is read from the browser's paint timing entry. Library
readiness is an upper bound observed through WebDriver, including polling and
driver overhead. The updater returns a local HTTP 204 response; these timings
do not measure GitHub network latency. Sequential runs have warmed filesystem
caches and are not cold-boot benchmarks.

The candidate rejects sustained minimized CPU usage above 1% of the machine.
Memory changes are reported rather than treated as proof of a leak: WebView2,
allocator, and query caches may retain memory. Short samples cannot establish
multi-day stability or prove absence of all races.

## Reproductions and focused checks

- Twenty-four simultaneous profile saves reproduced an atomic replacement
  failure before serialization. Store access now holds a thread mutex and an
  OS file lock across read, modification, BOM normalization, and replacement.
- Focus lifecycle tests cover late subscription completion after unmount and
  a stale initial focus response arriving after a newer event.
- Recovery and discovery polling stop while the application is unfocused;
  backend game-session observation remains active.
- Diagnostic comparisons read and parse each referenced file once per check,
  preserving duplicate occurrences and binding identities. A regression test
  supplies changing file revisions to ensure all operations use one read.
- The manual `diagnostic_large_profile_benchmark` test checks 400 parameters
  in a 4,001-line file. On the local Windows debug build, three samples changed
  from 4,254/3,960/3,984 ms to 15/11/11 ms. This measures that diagnostic workload,
  not whole-application startup or release-build performance.

Runtime results are available in the `windows-performance-results` artifact.
