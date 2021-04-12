(function() {var implementors = {};
implementors["mmtk"] = [{"text":"impl Sync for Address","synthetic":true,"types":[]},{"text":"impl Sync for ObjectReference","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for Allocators&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for AllocatorSelector","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for BumpAllocator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for DumpLinearScan","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for LargeObjectAllocator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MallocAllocator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for GcCounter","synthetic":true,"types":[]},{"text":"impl Sync for ObjectCounter","synthetic":true,"types":[]},{"text":"impl Sync for PerSizeClassObjectCounter","synthetic":true,"types":[]},{"text":"impl Sync for GcHookWork","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for AnalysisManager&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for FinalizableProcessor","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; Sync for Finalization&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; Sync for ForwardFinalization&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Sync for HeaderByte","synthetic":true,"types":[]},{"text":"impl Sync for FragmentedMapper","synthetic":true,"types":[]},{"text":"impl Sync for Map64","synthetic":true,"types":[]},{"text":"impl Sync for CommonFreeListPageResource","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for FreeListPageResource&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for HeapMeta","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MonotonePageResource&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for MonotonePageResourceConditional","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for CommonPageResource&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for SpaceDescriptor","synthetic":true,"types":[]},{"text":"impl Sync for VMRequest","synthetic":true,"types":[]},{"text":"impl Sync for IntArrayFreeList","synthetic":true,"types":[]},{"text":"impl Sync for NurseryZeroingOptions","synthetic":true,"types":[]},{"text":"impl Sync for PlanSelector","synthetic":true,"types":[]},{"text":"impl Sync for Options","synthetic":true,"types":[]},{"text":"impl Sync for RawMemoryFreeList","synthetic":true,"types":[]},{"text":"impl Sync for ReferenceProcessors","synthetic":true,"types":[]},{"text":"impl Sync for Semantics","synthetic":true,"types":[]},{"text":"impl Sync for SanityChecker","synthetic":true,"types":[]},{"text":"impl&lt;P, W&gt; Sync for ScheduleSanityGC&lt;P, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;P, W&gt; Sync for SanityPrepare&lt;P, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;P, W&gt; Sync for SanityRelease&lt;P, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; !Sync for SanityGCProcessEdges&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for SideMetadataScope","synthetic":true,"types":[]},{"text":"impl Sync for SideMetadataSpec","synthetic":true,"types":[]},{"text":"impl Sync for EventCounter","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Sync for LongCounter&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as Diffable&gt;::Val: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Sync for SizeCounter","synthetic":true,"types":[]},{"text":"impl Sync for MonotoneNanoTime","synthetic":true,"types":[]},{"text":"impl Sync for SharedStats","synthetic":true,"types":[]},{"text":"impl Sync for Stats","synthetic":true,"types":[]},{"text":"impl Sync for SynchronizedCounter","synthetic":true,"types":[]},{"text":"impl Sync for TreadMill","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MMTK&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for BarrierSelector","synthetic":true,"types":[]},{"text":"impl Sync for WriteTarget","synthetic":true,"types":[]},{"text":"impl Sync for NoBarrier","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; Sync for ObjectRememberingBarrier&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;E as ProcessEdgesWork&gt;::VM: VMBinding,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for ControllerCollectorContext&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for NoCopy&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for GcStatus","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for BaseUnsync&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for CommonUnsync&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for AllocationSemantics","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MutatorConfig&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; !Sync for Mutator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for PlanConstraints","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for GenCopy&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MarkSweep&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for ALLOCATOR_MAPPING","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for NoGC&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for SemiSpace&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for CommonSpace&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for SpaceOptions","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for LockFreeImmortalSpace&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; Sync for MallocSpace&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl Sync for ACTIVE_CHUNKS","synthetic":true,"types":[]},{"text":"impl Sync for ALLOC_MAP","synthetic":true,"types":[]},{"text":"impl Sync for MARK_MAP","synthetic":true,"types":[]},{"text":"impl&lt;C&gt; !Sync for CoordinatorMessage&lt;C&gt;","synthetic":true,"types":[]},{"text":"impl Sync for SchedulerStat","synthetic":true,"types":[]},{"text":"impl Sync for WorkStat","synthetic":true,"types":[]},{"text":"impl Sync for WorkerLocalStat","synthetic":true,"types":[]},{"text":"impl Sync for WorkBucketStage","synthetic":true,"types":[]},{"text":"impl !Sync for WorkerLocalPtr","synthetic":true,"types":[]},{"text":"impl&lt;C&gt; Sync for WorkerGroup&lt;C&gt;","synthetic":true,"types":[]},{"text":"impl Sync for ScheduleCollection","synthetic":true,"types":[]},{"text":"impl&lt;P, W&gt; Sync for Prepare&lt;P, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; !Sync for PrepareMutator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;W&gt; Sync for PrepareCollector&lt;W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;P, W&gt; Sync for Release&lt;P, W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;VM&gt; !Sync for ReleaseMutator&lt;VM&gt;","synthetic":true,"types":[]},{"text":"impl&lt;W&gt; Sync for ReleaseCollector&lt;W&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;W: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;ScanEdges&gt; Sync for StopMutators&lt;ScanEdges&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;ScanEdges: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Sync for EndOfGC","synthetic":true,"types":[]},{"text":"impl&lt;Edges&gt; Sync for ScanStackRoots&lt;Edges&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Edges: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;Edges&gt; !Sync for ScanStackRoot&lt;Edges&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Edges&gt; Sync for ScanVMSpecificRoots&lt;Edges&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Edges: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; !Sync for ProcessEdgesBase&lt;E&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Edges&gt; Sync for ScanObjects&lt;Edges&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Edges: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; Sync for ProcessModBuf&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Sync for OpaquePointer","synthetic":false,"types":[]},{"text":"impl Sync for UnsafeOptionsWrapper","synthetic":false,"types":[]},{"text":"impl Sync for ReferenceProcessor","synthetic":false,"types":[]},{"text":"impl&lt;VM:&nbsp;VMBinding&gt; Sync for BasePlan&lt;VM&gt;","synthetic":false,"types":[]},{"text":"impl&lt;VM:&nbsp;VMBinding&gt; Sync for CommonPlan&lt;VM&gt;","synthetic":false,"types":[]},{"text":"impl Sync for SFTMap","synthetic":false,"types":[]},{"text":"impl&lt;VM:&nbsp;VMBinding&gt; Sync for CopySpace&lt;VM&gt;","synthetic":false,"types":[]},{"text":"impl&lt;VM:&nbsp;VMBinding&gt; Sync for ImmortalSpace&lt;VM&gt;","synthetic":false,"types":[]},{"text":"impl&lt;VM:&nbsp;VMBinding&gt; Sync for LargeObjectSpace&lt;VM&gt;","synthetic":false,"types":[]},{"text":"impl&lt;C:&nbsp;Context&gt; Sync for Scheduler&lt;C&gt;","synthetic":false,"types":[]},{"text":"impl&lt;C:&nbsp;Context&gt; Sync for Worker&lt;C&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()