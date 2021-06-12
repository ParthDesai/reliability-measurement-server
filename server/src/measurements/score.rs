use crate::measurements::challenges::{CPUChallengeConfiguration, NetworkChallengeConfiguration};

fn find_mean(data: &Vec<u128>) -> u128 {
    let mut sum: u128 = 0;

    for element in data {
        sum += *element;
    }

    sum / (data.len() as u128)
}

/// calculate_score calculates score by finding mean of both cpu challenge data
/// and network challenge data. Both then mapped to domain of 0-50 and sum of both mapping
/// is substrated from 100 to obtain final score.
pub(crate) fn calculate_score(
    cpu_challenge_config: &CPUChallengeConfiguration,
    cpu_results: &Vec<u128>,
    network_challenge_config: &NetworkChallengeConfiguration,
    network_results: &Vec<u128>,
) -> u128 {
    let cpu_results_median = find_mean(cpu_results);
    let network_results_median = find_mean(network_results);

    // if any test took more than `max_milliseconds` we reject the client
    for cpu_result in cpu_results {
        if *cpu_result > cpu_challenge_config.max_milliseconds {
            return 0;
        }
    }
    for network_result in network_results {
        if *network_result > network_challenge_config.max_milliseconds {
            return 0;
        }
    }

    // Transform mean to 0-50 range
    let cpu_score = if cpu_results_median < cpu_challenge_config.ideal_milliseconds {
        0
    } else {
        ((cpu_results_median - cpu_challenge_config.ideal_milliseconds) * (50 - 0))
            / (cpu_challenge_config.max_milliseconds - cpu_challenge_config.ideal_milliseconds)
    };

    let network_score = if network_results_median < network_challenge_config.ideal_milliseconds {
        0
    } else {
        ((network_results_median - network_challenge_config.ideal_milliseconds) * (50 - 0))
            / (network_challenge_config.max_milliseconds
                - network_challenge_config.ideal_milliseconds)
    };

    // We need to subtract our score from 100 because score we calculated is using domain mapping and
    // in descending order.
    100 - (cpu_score + network_score)
}

#[cfg(test)]
mod tests {
    use crate::measurements::challenges::{
        CPUChallengeConfiguration, NetworkChallengeConfiguration,
    };
    use crate::measurements::score::calculate_score;

    #[test]
    fn test_score_calculation() {
        let cpu_challenge_config = CPUChallengeConfiguration {
            squarings: 0,
            ideal_milliseconds: 100,
            max_milliseconds: 1100,
        };

        let network_challenge_config = NetworkChallengeConfiguration {
            data_size_kb: 0,
            ideal_milliseconds: 200,
            max_milliseconds: 2200,
        };

        let cpu_results: Vec<u128> = vec![200, 300, 200, 500];
        let network_results: Vec<u128> = vec![300, 400, 300, 600];

        let score = calculate_score(
            &cpu_challenge_config,
            &cpu_results,
            &network_challenge_config,
            &network_results,
        );
        assert_eq!(score, 100 - (10 + 5));
    }

    #[test]
    fn test_score_calculation_edge_cases() {
        let cpu_challenge_config = CPUChallengeConfiguration {
            squarings: 0,
            ideal_milliseconds: 100,
            max_milliseconds: 1100,
        };

        let network_challenge_config = NetworkChallengeConfiguration {
            data_size_kb: 0,
            ideal_milliseconds: 200,
            max_milliseconds: 2200,
        };

        // 1200 is outside max_milliseconds range, so we reject
        // the client.
        let cpu_results: Vec<u128> = vec![1200, 300, 200, 500];
        let network_results: Vec<u128> = vec![300, 400, 300, 600];

        let score = calculate_score(
            &cpu_challenge_config,
            &cpu_results,
            &network_challenge_config,
            &network_results,
        );
        assert_eq!(score, 0);

        // CPU results median would be less than ideal_miliseconds, in that case it is 50 out of 50.
        let cpu_results: Vec<u128> = vec![1, 2, 3, 4];
        let network_results: Vec<u128> = vec![300, 400, 300, 600];

        let score = calculate_score(
            &cpu_challenge_config,
            &cpu_results,
            &network_challenge_config,
            &network_results,
        );
        assert_eq!(score, 100 - (0 + 5));
    }
}
