# Project overview

I built a comprehensive tutorial demonstrating how to create **inheritance vaults** using Miden Notes. The tutorial guides developers through building a complete system where crypto owners can lock assets in time-locked vaults that automatically transfer to designated beneficiaries after a deadline, while allowing owners to extend deadlines to prove they're still alive.

I chose this project because it addresses a critical real-world issue: Billions of dollars in cryptocurrency are permanently lost when holders die without proper inheritance mechanisms.

In my personal view, this idea was/is perfectly suitable because of the following:

1. **Solves a real-world use case**: Addresses the massive problem of lost crypto inheritance
2. **Easy to understand**: The problem statement, as well as the solution are not complex
3. **Limited code complexity**: Only minimal code required to demonstrate core concepts without overwhelming beginners
4. **Showcases key Miden features**: Miden Assembly and the power of programmable Notes for complex financial logic

The major development challenge was with the Web SDK, which led me to use the Rust client instead. Specific Web SDK issues that I encountered included:

1. **Caching account authorizations**: I could not get persistent authorization working properly when working with multiple accounts. Some transactions did not have the right sender.
2. **Deploying Smart Contract code**: Initially I tried to work with actual Smart Contract code instead of Notes, but I could not get deployment of Smart Contracts to work using the web client. Hence why I opted for the Notes-based approach
3. **CORS issues**: On the newest version of the web client, I consistently faced cross-origin problems, which you are already aware of.

On the other hand, working with the Rust SDK was a true blessing, I did not encounter any issues there!
