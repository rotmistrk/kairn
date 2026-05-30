# Pack Layout algorithm

Pack owns collection of PackLayoutItem

PackLayoutItem has:
 - minSize
 - maxSize
 - stretch
 - priority
 - gravity (Left|Right|Center)

Here:
 - minSze and maxSize are taken from the View size hints
 - rest are explicitely set

Following views may be used as children:
 - KeyItem has label; label length is minSize and maxSize
 - CmdItem - same
 - Label has text; length of the text is minSize and maxSize (used for status indicators and messages)
 - InputLine has size, it is used as minSize , maxSize unbounded
 - PrefixItem has label and it's minSize is the label
   - maxSize though depends on the state - according to what 'embedded' elements are visible - see general rule below
 - Generally group minSize is sum of all children minSizes, and maxSize is sum of all children maxSizes (unbounded if there is unbounded child)

PrefixItem logic: 
  1. when dormant
     - minsize=maxsize = label length; no visible children
  2. when active - uses pack layout for children

Cascading pack layout should do following:
1. Take list of minWidths of all visible children, sort by priority desc
2. Going from the top of the list (highest priority), get running sum as walking
3. If running sum + next item sceed layut width, stop, drop the rest from rendering)
   3.1. If it is boundary case - next (dropped) view is the same priority as last (included), smart layout may check all same priority views and 'pack' them to fit the size best, but this is enhancement
   3.2. Render thouse that are 'in' as below
4. With the list of viwes to show, check delta, and spread it proportionally to the 'stretch' values between the views that have non-0 stretch, not exceeding their maxSize.
5. With these sizes, let group render the 'visible' views and groups as normal, according to gravity and tree positions.

This document does NOT address how to construct the layout, should it be single or hierarchical.

