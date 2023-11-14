(module
  (type (;0;) (func (param i32) (result i32)))
  (type (;1;) (func))
  (func (;0;) (type 0) (param i32) (result i32)
    (local i32 i32 i32)
    i32.const 1
    local.set 1
    block  ;; label = @1
      local.get 0
      i32.const 2
      i32.lt_s
      br_if 0 (;@1;)
      local.get 0
      i32.const -1
      i32.add
      local.tee 1
      i32.const 7
      i32.and
      local.set 2
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.const -2
          i32.add
          i32.const 7
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 1
          local.set 0
          i32.const 1
          local.set 1
          br 1 (;@2;)
        end
        local.get 1
        i32.const -8
        i32.and
        local.set 3
        i32.const 1
        local.set 0
        i32.const 1
        local.set 1
        loop  ;; label = @3
          local.get 1
          local.get 0
          i32.add
          local.tee 0
          local.get 1
          i32.add
          local.tee 1
          local.get 0
          i32.add
          local.tee 0
          local.get 1
          i32.add
          local.tee 1
          local.get 0
          i32.add
          local.tee 0
          local.get 1
          i32.add
          local.tee 1
          local.get 0
          i32.add
          local.tee 0
          local.get 1
          i32.add
          local.set 1
          local.get 3
          i32.const -8
          i32.add
          local.tee 3
          br_if 0 (;@3;)
        end
      end
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.set 3
      loop  ;; label = @2
        local.get 3
        local.get 0
        i32.add
        local.set 1
        local.get 3
        local.set 0
        local.get 1
        local.set 3
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    local.get 1)
  (func (;1;) (type 1))
  (func (;2;) (type 1)
    call 1
    call 1)
  (func (;3;) (type 0) (param i32) (result i32)
    local.get 0
    call 0
    call 2)
  (table (;0;) 1 1 funcref)
  (memory (;0;) 16)
  (global (;0;) (mut i32) (i32.const 1048576))
  (export "memory" (memory 0))
  (export "fib" (func 3)))
