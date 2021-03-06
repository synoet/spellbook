import { FastifyInstance } from 'fastify';
import swagger from 'fastify-swagger';

export default function configureSwagger(app: FastifyInstance){
    const spec = {
      routePrefix: '/documentation',
      swagger: {
        info: {
          title: 'Spellbook Fastify API',
          description: 'Auto-generated docs for Spellbock\'s fastify backend API',
          version: '0.1.0'
        },
        externalDocs: {
          url: 'https://spellbook.sh',
          description: 'Check out spellbook\'s official site here'
        },
        host: 'localhost',
        schemes: ['http'],
        consumes: ['application/json'],
        produces: ['application/json'],
        tags: [
          { name: 'command', description: 'Command related end-points' },
          { name: 'user', description: 'User related end-points' }
        ],
        definitions: {
          User: {
            type: 'object',
            required: ['id', 'email'],
            properties: {
              id: { type: 'string', format: 'uuid' },
              email: {type: 'string', format: 'email' }
            }
          },
          // TODO: finish this type definition
          Command: {
              type: 'object',
              required: [],
              properties: {

              }
          }
        },
        securityDefinitions: {
        }
      },
      uiConfig: {
        docExpansion: 'full',
        deepLinking: false
      },
      exposeRoute: true
  }

  app.register(swagger, spec);
}


