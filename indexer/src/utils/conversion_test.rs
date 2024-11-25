#[cfg(test)]
mod tests {
    use crate::utils::conversion::{convert_address_result, convert_contract_result};
    use serde_json::json;

    #[test]
    fn test_convert_contract_result() {
        let script = "0d64077b226e616d65223a22436f6d6d6974746565496e666f436f6e7472616374222c2267726f757073223a5b5d2c226665617475726573223a7b7d2c22737570706f727465647374616e6461726473223a5b5d2c22616269223a7b226d6574686f6473223a5b7b226e616d65223a22766572696679222c22706172616d6574657273223a5b5d2c2272657475726e74797065223a22426f6f6c65616e222c226f6666736574223a302c2273616665223a66616c73657d2c7b226e616d65223a2267657441646d696e222c22706172616d6574657273223a5b5d2c2272657475726e74797065223a2248617368313630222c226f6666736574223a31342c2273616665223a66616c73657d2c7b226e616d65223a2273657441646d696e222c22706172616d6574657273223a5b7b226e616d65223a2261646d696e222c2274797065223a2248617368313630227d5d2c2272657475726e74797065223a22426f6f6c65616e222c226f6666736574223a39322c2273616665223a66616c73657d2c7b226e616d65223a22757064617465222c22706172616d6574657273223a5b7b226e616d65223a226e656646696c65222c2274797065223a22427974654172726179227d2c7b226e616d65223a226d616e6966657374222c2274797065223a22537472696e67227d2c7b226e616d65223a2264617461222c2274797065223a22416e79227d5d2c2272657475726e74797065223a22566f6964222c226f6666736574223a3136382c2273616665223a66616c73657d2c7b226e616d65223a22736574496e666f222c22706172616d6574657273223a5b7b226e616d65223a2273656e646572222c2274797065223a2248617368313630227d2c7b226e616d65223a226e616d65222c2274797065223a22537472696e67227d2c7b226e616d65223a226c6f636174696f6e222c2274797065223a22537472696e67227d2c7b226e616d65223a2277656273697465222c2274797065223a22537472696e67227d2c7b226e616d65223a22656d61696c222c2274797065223a22537472696e67227d2c7b226e616d65223a22676974687562222c2274797065223a22537472696e67227d2c7b226e616d65223a2274656c656772616d222c2274797065223a22537472696e67227d2c7b226e616d65223a2274776974746572222c2274797065223a22537472696e67227d2c7b226e616d65223a226465736372697074696f6e222c2274797065223a22537472696e67227d2c7b226e616d65223a226c6f676f222c2274797065223a22537472696e67227d5d2c2272657475726e74797065223a22426f6f6c65616e222c226f6666736574223a3232342c2273616665223a66616c73657d2c7b226e616d65223a22676574496e666f222c22706172616d6574657273223a5b7b226e616d65223a2263616e646964617465222c2274797065223a2248617368313630227d5d2c2272657475726e74797065223a22416e79222c226f6666736574223a3434382c2273616665223a66616c73657d2c7b226e616d65223a22676574416c6c496e666f222c22706172616d6574657273223a5b5d2c2272657475726e74797065223a224172726179222c226f6666736574223a3530372c2273616665223a66616c73657d2c7b226e616d65223a2264656c657465496e666f222c22706172616d6574657273223a5b7b226e616d65223a2263616e646964617465222c2274797065223a2248617368313630227d5d2c2272657475726e74797065223a22426f6f6c65616e222c226f6666736574223a3538392c2273616665223a66616c73657d2c7b226e616d65223a225f696e697469616c697a65222c22706172616d6574657273223a5b5d2c2272657475726e74797065223a22566f6964222c226f6666736574223a3639342c2273616665223a66616c73657d5d2c226576656e7473223a5b5d7d2c227065726d697373696f6e73223a5b7b22636f6e7472616374223a22307837323663623665306364383632386131333530613631313338343638383931316162373566353162222c226d6574686f6473223a5b22726970656d64313630222c22736861323536225d7d2c7b22636f6e7472616374223a22307861636365366664383064343465313739366161306332633632356539653465306365333965666330222c226d6574686f6473223a5b22646573657269616c697a65222c2273657269616c697a65225d7d2c7b22636f6e7472616374223a22307865663430373361306632623330356133386563343035306534643364323862633430656136336635222c226d6574686f6473223a5b2267657443616e64696461746573225d7d2c7b22636f6e7472616374223a22307866666664633933373634646261646464393763343866323532613533656134363433666161336664222c226d6574686f6473223a5b22757064617465225d7d5d2c22747275737473223a5b5d2c226578747261223a7b22417574686f72223a224e454f222c22456d61696c223a22646576656c6f706572406e656f2e6f7267222c224465736372697074696f6e223a22546869732069732061204e656f3320436f6e7472616374227d7d0d03044e4546334e656f2e436f6d70696c65722e43536861727020332e302e30000000000000000000000000000000000000000000000000000000000000000000000000000000000006fda3fa4346ea532a258fc497ddaddb6437c9fdff067570646174650300000ff563ea40bc283d4d0e05c48ea305b3f2a07340ef0d67657443616e646964617465730000010f1bf575ab1189688413610a35a12886cde0b66c7209726970656d643136300100010f1bf575ab1189688413610a35a12886cde0b66c72067368613235360100010fc0ef39cee0e4e925c6c2a06a79e1440dd86fceac0973657269616c697a650100010fc0ef39cee0e4e925c6c2a06a79e1440dd86fceac0b646573657269616c697a650100010f0000fde702340e41f827ec8c4041f827ec8c405701000c0a737570657241646d696e342070684ad82403ca0014972610684ad824094aca001428033a22035822024057000178419bf667ce41925de83122024041925de83140419bf667ce40ca405700010c09466f7262696464656e34a441f827ec8c3417780c0a737570657241646d696e341211db2022024057000278aa2604793a405700027978419bf667ce41e63f18844041e63f1884405700033555ffffffaa26160c114e6f20617574686f72697a6174696f6e2e3a7a797837000040370000405700015978db308b408b40db304057080a0c09466f7262696464656e7841f827ec8c34943701007010db2071684a72ca731074221f6a6ccec14575766d34617707786f0797260a11db204a714522096c9c746c6b30e10c1753656e646572206973206e6f742043616e646964617465693546ffffff7f097f087f077e7d7c7b7a79781ac04a344b726a370400783573ffffff344211db20220240370100405702015a78db308b5b8b7068db2837030037020071694ad824094aca001428033a220240db30403702004037030040db2840570001405700027978419bf667ce41e63f18844041e63f1884403704004057010178350effffff341770684ad82403ca10b726086837050022050b22024057000178419bf667ce41925de83122024041925de83140370500405703005934287010c4007168419c08ed9c26176841f354bf1d726a11ce0b982607696a11cecf22e5692202405700011a78419bf667ce41df30b89a22024041df30b89a40419c08ed9c4041f354bf1d40cf405702010c09466f7262696464656e7841f827ec8c260711db2022073598fdffff351bfeffff78355ffeffff70783558feffff3561ffffff71694ad82403ca10b7260a68340d11db20220710db2022024057000178db28419bf667ce412f58c5ed40412f58c5ed40cf4056040c14c045430c6122560cbdc5868c3a4ce02f02ddbcc1600c020c21db30620c054156e7b327db30630c0177db3061409ae617b512c01f0c066465706c6f790c14fda3fa4346ea532a258fc497ddaddb6437c9fdff41627d5b52".to_string();

        let notifications = json!([
          {
            "contract": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
            "eventname": "Deploy",
            "state": {
              "type": "Array",
              "value": [
                {
                  "type": "ByteString",
                  "value": "4RvlQ9qY2B3u+HBeVhEMrbavdrc="
                }
              ]
            }
          },
          {
            "eventname": "OnDeploy",
            "state": {
              "value": []
            }
          }
        ]);

        let block_height = 210;

        let result = convert_contract_result(script, notifications, block_height);

        assert_eq!(result.len(), 1);
        let contract = &result[0];
        assert_eq!(contract.block_index, block_height);
        assert_eq!(contract.hash, "0xb776afb6ad0c11565e70f8ee1dd898da43e51be1");
        assert_eq!(contract.contract_type, "[]");
    }

    #[test]
    fn test_convert_address_result() {
        let notifications = json!([
            {
                "contract": "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5",
                "eventname": "Transfer",
                "state": {
                    "type": "Array",
                    "value": [
                        {
                            "type": "ByteString",
                            "value": "axI92L7HGGSIUrvHhZXjU2oFj58="
                        },
                        {
                            "type": "ByteString",
                            "value": "dVE6zv92GLfukg8P5gFa0cDxb/0="
                        },
                        {
                            "type": "Integer",
                            "value": "100000"
                        }
                    ]
                }
            },
            {
                "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
                "eventname": "Transfer",
                "state": {
                    "type": "Array",
                    "value": [
                        {
                            "type": "Any",
                            "value": null
                        },
                        {
                            "type": "ByteString",
                            "value": "axI92L7HGGSIUrvHhZXjU2oFj58="
                        },
                        {
                            "type": "Integer",
                            "value": "40300000"
                        }
                    ]
                }
            }
        ]);

        let block_height = 210;

        let result = convert_address_result(notifications, block_height);

        assert_eq!(result.len(), 2);

        let sender = &result[0];

        assert_eq!(sender.block_index, block_height);
        assert_eq!(sender.address, "NVg7LjGcUSrgxgjX3zEgqaksfMaiS8Z6e1");
        assert_eq!(sender.balances, "{}");

        let recipient = &result[1];
        assert_eq!(recipient.block_index, block_height);
        assert_eq!(recipient.address, "NWcHZ95TNzfVCfvK2AvY5xyEw6ur3oD3wL");
        assert_eq!(recipient.balances, "{}");
    }
}