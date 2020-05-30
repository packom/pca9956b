// https://www.jenkins.io/doc/book/pipeline/docker/
pipeline {
    agent {
        docker { image 'piersfinlayson/build-amd64:0.3.2' }
    }
    stages {
        stage('Clone') {
            steps {
                withCredentials([usernamePassword(credentialsId: 'github.packom', usernameVariable: 'USERNAME', passwordVariable: 'PASSWORD')]) {
                    sh '''
                        cd ~/builds && \
                        git clone https://packom:$PASSWORD@github.com/packom/pca9956b && \
                        cd pca9956b && \
                        echo `awk '/^version / {print $3;}' Cargo.toml | sed 's/"//g'` > /tmp/version && \
                        echo "Current version is:" && \
                        cat /tmp/version
                    '''
                }
            }
        }
        stage('Build') {
            steps {
                sh '''
                    cd ~/builds/pca9956b && \
                    cargo build
                '''
            }
        }
        stage('Test') {
            steps {
                sh '''
                    cd ~/builds/pca9956b && \
                    cargo test
                '''
            }
        }
        stage('Publish') {
            steps {
                withCredentials([usernamePassword(credentialsId: 'crates.packom', usernameVariable: 'USERNAME', passwordVariable: 'PASSWORD')]) {
                    sh '''
                        cd ~/builds/pca9956b && \
                        CURV=$(cat /tmp/version) && \
                        echo `cargo search pca9956b | awk '/^pca9956b / {print $3;}' | sed 's/"//g'` > /tmp/old_version && \
                        echo "Old version is:" && \
                        cat /tmp/old_version && \
                        OLDV=$(cat /tmp/old_version) && \
if [ $CURV != $OLDV ]
then
    cargo publish --token $PASSWORD 
else
    echo "No changes to publish"
fi
                    '''
                }
            }
        }
    }
}
