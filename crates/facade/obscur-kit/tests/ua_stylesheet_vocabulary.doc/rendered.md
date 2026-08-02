Renders a view through a real runtime with the UA sheet installed at
`SystemDefault`, and hands the settled runtime back.

The sheet has to be installed for the harvest as well as for the assertions:
`resolve_generated_content` only records a part once something asks it to.
