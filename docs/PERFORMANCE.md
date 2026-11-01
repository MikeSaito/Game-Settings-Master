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
  Contention with another instance returns an error rather than waiting for a
  potentially stalled process. The operation can be retried after it releases
  the lock.
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

## Windows results, 2026-11-01

[Audit run 41304104090](https://github.com/MikeSaito/Game-Settings-Master/actions/runs/41304104090)
passed on Windows Server 2022 with four logical processors. The baseline is
v1.5.3 (`92b5eab`); the measured candidate is `f2004f6`. The subsequent `a6870d9`
change makes an already-held cross-process store lock fail promptly; its
contention/recovery regression and full CI passed separately. The measured
workload contains no competing application instance.

| Phase | Baseline CPU | Candidate CPU | Baseline end private memory | Candidate end private memory |
| --- | ---: | ---: | ---: | ---: |
| Library idle | 0.185% | 0.214% | 118.4 MiB | 117.6 MiB |
| Minimized | 0.254% | 0.084% | 120.9 MiB | 117.5 MiB |
| Editor idle | 0.165% | 0.235% | 139.8 MiB | 146.8 MiB |
| After 20 preview/cancel cycles | 0.165% | 0.199% | 158.2 MiB | 156.5 MiB |

All percentages include the application and WebView2 and are percentages of the
whole machine. These are single paired runs: the small foreground differences
and the lower minimized CPU are observations, not an established speedup ratio.
Both builds already have low idle CPU usage in this workload.

Candidate private memory fell from 210.3 MiB at the first post-stress sample to
156.5 MiB after about 57 seconds. Its process count stayed at eight throughout.
Handles changed from 3,011 at the start of editor idle to 2,994 after stress;
threads changed from 134 to 128. Recovery/discovery IPC counters were empty while
minimized in both builds. The explicit focus gating additionally covers an
unfocused window that remains visible, which is checked by the frontend test.
The maximum foreground JavaScript timer delay during stress was 38.2 ms for the
baseline and 37.4 ms for the candidate; this is not a frame-rendering benchmark.

Document-to-library upper bounds were 0.77/1.57/1.36 seconds for the baseline
and 1.64/1.42/2.10 seconds for the candidate. Median first contentful paint was
1.14 and 1.23 seconds respectively. Three noisy WebDriver-observed startups do
not establish a startup improvement or a regression. In particular, the first
baseline driver launch took 9.6 seconds while its document-to-library time was
0.77 seconds, demonstrating why driver startup must not be reported as app time.

No sustained memory/process/handle accumulation was observed in this short
scenario. It does not cover multi-hour sessions, all real store libraries, or
GPU power/usage on physical hardware. The existing local discovery-cache test
also passed in 2.14 seconds, including its initial scan and cached lookup; that
is not an end-to-end application startup measurement.

Final code validation: 255 Rust tests passed (two existing/manual ignores),
243 frontend tests, 37 browser scenarios, 47 landing scenarios, both Clippy
modes, formatting, generated bindings, and native config/recovery tests passed.
