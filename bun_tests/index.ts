function calculateReward(i_zeros: number, i_stake: number, t_zeros: number, t_stake: number, block_reward: number) {
    const reward = ((i_zeros + i_stake) / (t_zeros + t_stake)) * block_reward;

    return Math.floor(reward).toLocaleString();
}

const block_reward = 1_0000000;

let total_zeros = 0;
let total_stake = 0;

const user_1_zeros = Math.pow(8, 1);
const user_2_zeros = Math.pow(8, 2);
const user_3_zeros = Math.pow(8, 3);

const user_1_stake = 10000;
const user_2_stake = 1000;
const user_3_stake = 100;

total_zeros += user_1_zeros;
total_zeros += user_2_zeros;
total_zeros += user_3_zeros;

total_stake += user_1_stake;
total_stake += user_2_stake;
total_stake += user_3_stake;

const rewards = [
    calculateReward(user_1_zeros, user_1_stake, total_zeros, total_stake, block_reward),
    calculateReward(user_2_zeros, user_2_stake, total_zeros, total_stake, block_reward),
    calculateReward(user_3_zeros, user_3_stake, total_zeros, total_stake, block_reward),
]

console.log(
    rewards,
    rewards.reduce((acc, curr) => acc + parseInt(curr.replace(/\D/g, '')), 0).toLocaleString()
)