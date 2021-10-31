use std::str::FromStr;
use mojaloop_api::clients::FspiopClient;
use futures::{SinkExt,StreamExt};
use voodoo_doll::*;
use mojaloop_api::clients;
use mojaloop_api::central_ledger::{settlement_models, participants};
use mojaloop_api::fspiox_api::{FspId,Currency,Amount};

async fn create_hub_account(
    client: &mut mojaloop_api::clients::central_ledger::Client,
    currency: Currency,
    r#type: participants::HubAccountType
) -> fspiox_api::clients::Result<()> {
    let request = participants::PostHubAccount {
        // TODO: parametrise hub name?
        name: FspId::from("Hub").unwrap(),
        account: participants::HubAccount {
            r#type,
            currency,
        }
    };

    client.send(request).await.and(Ok(()))
}

// TODO: this is slightly difficult because the image tag being deployed is generated
// #[tokio::test]
// async fn test_create_destroy() {
//     create(None, &None).await.expect("create okay");
//     destroy(None, &None).await.expect("destroy okay");
// }

#[tokio::test]
async fn test_switch_setup() {
    let mut ml_central_ledger = clients::central_ledger::Client::from_k8s_defaults().await.expect("central ledger connection okay");

    let currency = Currency::AOR;

    create_hub_account(
        &mut ml_central_ledger,
        currency,
        participants::HubAccountType::HubMultilateralSettlement
    ).await.expect("hub settlement account created");

    create_hub_account(
        &mut ml_central_ledger,
        currency,
        participants::HubAccountType::HubReconciliation
    ).await.expect("hub reconciliation account created");

    let request = settlement_models::PostSettlementModel {
        settlement_model: settlement_models::SettlementModel {
            auto_position_reset: true,
            ledger_account_type: settlement_models::LedgerAccountType::Position,
            settlement_account_type: settlement_models::SettlementAccountType::Settlement,
            name: settlement_models::SettlementModelName::from("DEFERREDNET").unwrap(),
            require_liquidity_check: true,
            settlement_delay: settlement_models::SettlementDelay::Deferred,
            settlement_granularity: settlement_models::SettlementGranularity::Net,
            settlement_interchange: settlement_models::SettlementInterchange::Multilateral,
            currency,
        }
    };
    ml_central_ledger.send(request).await.expect("settlement model created");

    ml_central_ledger.send(
        participants::PostParticipant {
            participant: participants::NewParticipant {
                name: FspId::from("payerfsp").unwrap(),
                currency,
            },
        }
    ).await.expect("payerfsp created");

    ml_central_ledger.send(
        participants::PostParticipant {
            participant: participants::NewParticipant {
                name: FspId::from("payeefsp").unwrap(),
                currency,
            },
        }
    ).await.expect("payeefsp created");

    ml_central_ledger.send(
        participants::PostInitialPositionAndLimits {
            name: FspId::from("payerfsp").unwrap(),
            initial_position_and_limits: participants::InitialPositionAndLimits {
                currency,
                limit: participants::Limit {
                    r#type: participants::LimitType::NetDebitCap,
                    value: 10000,
                },
                initial_position: Amount::ZERO,
            }
        }
    ).await.expect("set payerfsp initial position and limits");

    ml_central_ledger.send(
        participants::PostInitialPositionAndLimits {
            name: FspId::from("payeefsp").unwrap(),
            initial_position_and_limits: participants::InitialPositionAndLimits {
                currency,
                limit: participants::Limit {
                    r#type: participants::LimitType::NetDebitCap,
                    value: 10000,
                },
                initial_position: Amount::ZERO,
            }
        }
    ).await.expect("set payeefsp initial position and limits");

    let (mut voodoo_write, mut voodoo_read) = get_pod_stream(None).await.expect("voodoo doll stream connected").split();

    let transfer_id = mojaloop_api::fspiox_api::transfer::TransferId(mojaloop_api::fspiox_api::CorrelationId::new());

    voodoo_write.send(
        Message::Text(
            serde_json::to_string(
                &protocol::ClientMessage {
                    id: transfer_id.0,
                    content: protocol::Request::Transfer(
                        protocol::TransferMessage {
                            msg_sender: FspId::from("payerfsp").unwrap(),
                            msg_recipient: FspId::from("payeefsp").unwrap(),
                            currency,
                            amount: Amount::from_str("1000").unwrap(),
                            transfer_id,
                        }
                    ),
                }
            ).unwrap()
        )
    ).await.expect("sent message to voodoo server");

    while let Some(msg) = voodoo_read.next().await {
        let msg = msg.expect("message present");
        match msg {
            Message::Text(s) => {
                let response_msg: protocol::ServerMessage =
                    serde_json::from_str(&s).expect("message from server deserialized");
                // TODO: we should be checking the ID of the notification
                match response_msg.content {
                    protocol::Notification::TransferComplete(tc) => {
                        if tc.id == transfer_id {
                            println!("Transfer complete. ID: {}", transfer_id);
                            break;
                        }
                    }
                    protocol::Notification::TransferError(te) => {
                        if te.id == transfer_id {
                            println!("Transfer error. Error: {:?}", s);
                            break;
                        }
                    }
                    _ => {
                        // Ignore anything else; we're not interested
                    }
                }
            }
            _ => {
                println!("Incoming non-text:");
                println!("{}", msg);
            }
        }
    }

    // Cleanup
    voodoo_write.close().await.expect("close okay");
}
