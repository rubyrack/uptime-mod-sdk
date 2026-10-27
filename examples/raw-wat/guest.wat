;; A script mod written without the Rust SDK or the Lua guest: a core wasm
;; module for the world `uptime:mod/uptime-mod@0.1.0` (mod-sdk/wit/uptime-mod.wit).
;; It does three things each step:
;;
;;   1. Reads `quantum` out of the snapshot envelope (named msgpack).
;;   2. Issues one command, AcceptProspect { prospect_id: <quantum> }.
;;   3. Saves state: one byte, the number of steps it has run.
;;
;; `init` restores that byte from the saved state, so the count survives a load.
;; A step with an empty snapshot (the host sends one between refreshes) issues
;; nothing and still counts.
;;
;; Build the component with wasm-tools:
;;   wasm-tools component embed mod-sdk/wit guest.wat --world uptime-mod -o embedded.wasm
;;   wasm-tools component new embedded.wasm -o mod.wasm
;;
;; Memory map: 0x000 return area; 0x200 the command (38 bytes); 0x300 the command
;; list (one pointer and length pair); 0x400 the state byte; 0x30000 the heap that
;; cabi_realloc hands out, reset after every call.
(module
  (memory (export "memory") 4)
  (global $heap (mut i32) (i32.const 0x30000))
  (global $steps (mut i32) (i32.const 0))

  ;; fixmap(1) fixstr(14) "AcceptProspect" fixmap(1) fixstr(11) "prospect_id" uint64.
  ;; The 8 value bytes go at 0x21e, big endian.
  (data (i32.const 0x200) "\81\aeAcceptProspect\81\abprospect_id\cf")

  ;; The canonical ABI asks the guest for memory to put arguments in.
  (func (export "cabi_realloc") (param $old i32) (param $old_size i32) (param $align i32)
    (param $size i32) (result i32)
    (local $p i32) (local $end i32) (local $have i32)
    (local.set $p
      (i32.and
        (i32.add (global.get $heap) (i32.sub (local.get $align) (i32.const 1)))
        (i32.sub (i32.const 0) (local.get $align))))
    (local.set $end (i32.add (local.get $p) (local.get $size)))
    (local.set $have (i32.shl (memory.size) (i32.const 16)))
    (if (i32.gt_u (local.get $end) (local.get $have))
      (then
        (if (i32.eq
              (memory.grow
                (i32.add (i32.shr_u (i32.sub (local.get $end) (local.get $have)) (i32.const 16))
                  (i32.const 1)))
              (i32.const -1))
          (then unreachable))))
    (global.set $heap (local.get $end))
    (local.get $p))

  (func $reset (param i32) (global.set $heap (i32.const 0x30000)))
  (export "cabi_post_init" (func $reset))
  (export "cabi_post_step" (func $reset))
  (export "cabi_post_migrate" (func $reset))

  ;; Reads the msgpack unsigned integer at $p.
  (func $uint (param $p i32) (result i64)
    (local $tag i32) (local $n i32) (local $acc i64)
    (local.set $tag (i32.load8_u (local.get $p)))
    (if (i32.lt_u (local.get $tag) (i32.const 0x80))
      (then (return (i64.extend_i32_u (local.get $tag)))))
    ;; 0xcc u8, 0xcd u16, 0xce u32, 0xcf u64: 1, 2, 4 or 8 bytes follow.
    (local.set $n
      (i32.shl (i32.const 1) (i32.sub (local.get $tag) (i32.const 0xcc))))
    (local.set $p (i32.add (local.get $p) (i32.const 1)))
    (loop $byte
      (local.set $acc
        (i64.or (i64.shl (local.get $acc) (i64.const 8))
          (i64.load8_u (local.get $p))))
      (local.set $p (i32.add (local.get $p) (i32.const 1)))
      (local.set $n (i32.sub (local.get $n) (i32.const 1)))
      (br_if $byte (local.get $n)))
    (local.get $acc))

  ;; The address of the value after the key "quantum" in the snapshot, or 0.
  ;; The key is fixstr(7): the bytes a7 'q' 'u' 'a' 'n' 't' 'u' 'm'.
  (func $find_quantum (param $snap i32) (param $len i32) (result i32)
    (local $i i32)
    (if (i32.lt_u (local.get $len) (i32.const 9)) (then (return (i32.const 0))))
    (loop $scan
      (if (i64.eq
            (i64.load align=1 (i32.add (local.get $snap) (local.get $i)))
            (i64.const 0x6d75746e617571a7))
        (then (return (i32.add (i32.add (local.get $snap) (local.get $i)) (i32.const 8)))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $scan (i32.lt_u (local.get $i) (i32.sub (local.get $len) (i32.const 8)))))
    (i32.const 0))

  ;; init(config, saved-state) -> result<_, string>
  ;; Arguments: config ptr, config len, option tag, saved ptr, saved len.
  (func (export "init") (param $cfg i32) (param $cfg_len i32)
    (param $has i32) (param $saved i32) (param $saved_len i32) (result i32)
    (if (i32.and (local.get $has) (i32.ne (local.get $saved_len) (i32.const 0)))
      (then (global.set $steps (i32.load8_u (local.get $saved)))))
    (i32.store8 (i32.const 0) (i32.const 0))
    (i32.const 0))

  ;; step(quantum, snapshot, events) -> result<step-output, string>
  ;; Arguments: index, sim-time-ns, seed, snapshot ptr and len, events ptr and len.
  (func (export "step") (param $index i64) (param $time i64) (param $seed i64)
    (param $snap i32) (param $snap_len i32) (param $ev i32) (param $ev_len i32) (result i32)
    (local $at i32) (local $q i64) (local $i i32) (local $count i32)
    (global.set $steps (i32.add (global.get $steps) (i32.const 1)))
    (local.set $at (call $find_quantum (local.get $snap) (local.get $snap_len)))
    (if (local.get $at)
      (then
        (local.set $q (call $uint (local.get $at)))
        ;; The value, one byte at a time from the low end of 0x21e..0x225.
        (local.set $i (i32.const 7))
        (loop $put
          (i32.store8 (i32.add (i32.const 0x21e) (local.get $i))
            (i32.wrap_i64 (local.get $q)))
          (local.set $q (i64.shr_u (local.get $q) (i64.const 8)))
          (local.set $i (i32.sub (local.get $i) (i32.const 1)))
          (br_if $put (i32.ge_s (local.get $i) (i32.const 0))))
        (i32.store (i32.const 0x300) (i32.const 0x200))
        (i32.store (i32.const 0x304) (i32.const 38))
        (local.set $count (i32.const 1))))
    (i32.store8 (i32.const 0x400) (global.get $steps))
    ;; ok(step-output { commands, panels: [], toasts: [], state: some([steps]) })
    (i32.store8 (i32.const 0) (i32.const 0))
    (i32.store (i32.const 4) (i32.const 0x300))
    (i32.store (i32.const 8) (local.get $count))
    (i32.store (i32.const 12) (i32.const 0))
    (i32.store (i32.const 16) (i32.const 0))
    (i32.store (i32.const 20) (i32.const 0))
    (i32.store (i32.const 24) (i32.const 0))
    (i32.store8 (i32.const 28) (i32.const 1))
    (i32.store (i32.const 32) (i32.const 0x400))
    (i32.store (i32.const 36) (i32.const 1))
    (i32.const 0))

  ;; migrate(from-schema, state) -> result<list<u8>, string>: the state is kept as is.
  (func (export "migrate") (param $from i32) (param $state i32) (param $len i32) (result i32)
    (i32.store8 (i32.const 0) (i32.const 0))
    (i32.store (i32.const 4) (local.get $state))
    (i32.store (i32.const 8) (local.get $len))
    (i32.const 0)))
