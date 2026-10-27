# Money, Custody and Notes

This guide describes the M10d money work implemented through slices EF–EI. The exact scope and
remaining work are tracked in [the project plan](../../PROJECT_PLAN.md#7-milestones). Money is not a
world-level currency switch: households decide what to hold and accept from what they have bought,
what sellers accept, and what they expect a payment to buy.

## Household valuation and acceptance

At midnight, the simulation builds a household-specific table of the goods its money bought. The
table avoids scanning every transaction during each decision. Alongside this estimate, a household
keeps a belief about whether others will accept the money. Payments received and refusals by sellers
who offer wanted goods affect that belief. A household tends to keep money for spending and for
expected obligations to a common store. When a seller lacks the coin needed for a levy, the seller
can offer goods for that coin. Fines and ground rent owed to the store are reserved and paid in the
coin the store accepts. Market summaries report sellers accepting each money and households that
hold it.

This is inferred purchasing power and acceptance, not a prescripted currency tier or guaranteed
price stability. Household decisions use local experience; the observer summarizes the resulting
state.

## Custody and receipts

Some households wait for a home or have more goods than their home can hold. A household may leave
surplus goods with close kin who have room, without charge, or with the nearest neighbor who has
room, for a small monthly fee. Custody changes where goods are kept, not who owns them. The goods
return to the household when its new home has room. A literate keeper can write a receipt that
records whose goods are held. Market summaries show custody relationships.

## Redeemable notes

A literate keeper can issue notes against goods deposited with them. Each note represents one
kilogram of its named good, is held by the depositor, and is redeemable at the keeper's door. The
holder presents notes to withdraw the backing goods. The keeper pays in a drawn order from the goods
left in custody; if backing is short, the unpaid presentation is refused and remains owed. The
observer reports notes issued, backing, and refusals for each keeper.

A note's local value reflects the good it can buy, the share of presented notes actually paid, and
the cost of walking to redeem it. Holders may present notes when they need the good or hear of a
refusal. This makes redemption risk and distance part of household valuation.

## Lending, runs and suspension

A keeper may lend additional notes when its backing remains at least 40 percent of what it owes on
notes. The borrower owes the redemption good, and the keeper books the loan. Presentations draw
first on goods left with the keeper, then on the keeper's own goods beyond its reserves. A keeper's
backing measure includes both sources, while keeping the loan obligation visible in the books.

Holders present their notes after hearing that the keeper refused a presentation or that a borrower
defaulted, or after seeing a substantial share presented in a week. If the keeper refuses, it
suspends redemption for 30 days unless it becomes fully covered again. During suspension, it refuses
new presentations. These rules create a possible run and a clear observable shortfall; they do not
force a keeper to fail or a community to rescue it.

## Bearer-heard rule

The *bearer heard* policy makes refused notes a debt the keeper owes to their holder. When the rule
applies, the refused notes leave the holder's purse and are recorded as a claim in the keeper's
credit book, avoiding counting the same obligation twice. A gathering can hear the claim through
the existing debt process. Without the rule, the holder keeps the refused notes and the claim is not
heard through that process. Market summaries expose notes written as loans and the suspension end
date.

## Persistence and scope

M10d advances the schema boundary to wire 1.106, save schema 214 and content API 195. The notes,
custody, issue backing, refused claims and suspension state are saved so a loaded world continues
with the same obligations. Generated schema code is not hand-edited; schema changes follow the
append-only rules in [AGENTS.md](../../AGENTS.md) and the persistence decisions.

Implemented here: household-level value and acceptance, custody receipts, goods-backed notes,
bounded lending, redemption runs, suspension and debt hearings under *bearer heard*. The project
plan lists the remaining questions and planned work, including polity-issued notes, cross-polity
acceptance, counterfeiting and the complete M10 demonstration. These mechanisms demonstrate modeled
behavior and engineering invariants; they do not establish historical realism.
